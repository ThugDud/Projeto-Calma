use crate::config::{
    Config, MAX_NOTIFICATION_ADVANCE_MINUTES, MAX_POLL_INTERVAL_SECS,
    MIN_NOTIFICATION_ADVANCE_MINUTES, MIN_POLL_INTERVAL_SECS,
};
use crate::desktop::notifications::DesktopDiagnostics;
use crate::errors::AppResult;
use crate::models::event::Event;
use crate::models::project::Project;
use crate::models::task::{Task, TaskPriority, TaskStatus};
use crate::services::date_parser::{
    parse_pt_date, parse_quick_event, parse_quick_task, parse_time,
};
use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::Connection;
use std::collections::HashSet;

/// Identificador único de um nó da hierarquia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HierarchyNodeId {
    Project(i64),
    Task(i64),
}

/// Item visível na árvore hierárquica.
#[derive(Debug, Clone)]
pub struct HierarchyItem {
    pub id: HierarchyNodeId,
    pub title: String,
    pub depth: usize,
    pub is_project: bool,
    pub has_children: bool,
    pub is_expanded: bool,
    pub status: Option<TaskStatus>,
    pub due_date: Option<NaiveDate>,
    pub priority: Option<TaskPriority>,
    pub progress: Option<(usize, usize)>,
}

/// Estado do formulário de edição de tarefa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEditState {
    pub task_id: i64,
    /// 0 = Título, 1 = Prazo, 2 = Prioridade, 3 = Projeto
    pub active_field: usize,
    pub title: String,
    pub due_date_str: String,
    pub priority: TaskPriority,
    pub project_id: Option<i64>,
}

/// Estado do formulário de edição de evento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEditState {
    pub event_id: i64,
    /// 0 = Título, 1 = Data, 2 = Horário
    pub active_field: usize,
    pub title: String,
    pub date_str: String,
    pub time_str: String,
}

/// Tipo de item que pode ser excluído.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteTarget {
    Task,
    Project,
    Event,
}

/// Modos de entrada de dados e diálogo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    QuickAdd,
    QuickAddEvent,
    ConfirmDelete {
        target: DeleteTarget,
        id: i64,
        title: String,
    },
    EditTask(TaskEditState),
    EditEvent(EventEditState),
    NewSubtask {
        parent_task_id: i64,
        parent_task_title: String,
        buffer: String,
    },
    NewProject {
        parent_project_id: Option<i64>,
        buffer: String,
    },
    EditProject {
        project_id: i64,
        buffer: String,
    },
}

/// Visualização ativa da aplicação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Calendario,
    Tarefas,
    Hierarquia,
    VisaoGeral,
    Eventos,
    Configuracoes,
}

/// Seções exibidas na tela de configurações.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Notifications,
    Desktop,
    Appearance,
}

impl SettingsSection {
    pub fn title(self) -> &'static str {
        match self {
            SettingsSection::General => "GERAL",
            SettingsSection::Notifications => "NOTIFICAÇÕES",
            SettingsSection::Desktop => "AMBIENTE GRÁFICO",
            SettingsSection::Appearance => "APARÊNCIA",
        }
    }
}

/// Identificadores das configurações editáveis.
///
/// A lista centraliza a ordem de navegação, a seção e a forma de leitura e
/// escrita de cada opção. Novas configurações podem ser adicionadas aqui sem
/// reescrever a tela inteira.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingId {
    StartWithSystem,
    BackgroundMonitoring,
    PollInterval,
    NotificationsEnabled,
    UpcomingTasks,
    OverdueTasks,
    UpcomingEvents,
    NotificationAdvance,
}

impl SettingId {
    pub const ALL: [SettingId; 8] = [
        SettingId::StartWithSystem,
        SettingId::BackgroundMonitoring,
        SettingId::PollInterval,
        SettingId::NotificationsEnabled,
        SettingId::UpcomingTasks,
        SettingId::OverdueTasks,
        SettingId::UpcomingEvents,
        SettingId::NotificationAdvance,
    ];

    pub fn section(self) -> SettingsSection {
        match self {
            SettingId::StartWithSystem
            | SettingId::BackgroundMonitoring
            | SettingId::PollInterval => SettingsSection::General,
            SettingId::NotificationsEnabled
            | SettingId::UpcomingTasks
            | SettingId::OverdueTasks
            | SettingId::UpcomingEvents
            | SettingId::NotificationAdvance => SettingsSection::Notifications,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SettingId::StartWithSystem => "Iniciar com o sistema",
            SettingId::BackgroundMonitoring => "Monitoramento em segundo plano",
            SettingId::PollInterval => "Intervalo de verificação",
            SettingId::NotificationsEnabled => "Notificações da área de trabalho",
            SettingId::UpcomingTasks => "Avisar sobre tarefas do dia",
            SettingId::OverdueTasks => "Avisar sobre tarefas atrasadas",
            SettingId::UpcomingEvents => "Avisar sobre eventos do dia e próximos",
            SettingId::NotificationAdvance => "Antecedência do aviso de evento",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            SettingId::StartWithSystem => {
                "A preferência é salva, mas a integração automática com o sistema ainda não está disponível."
            }
            SettingId::BackgroundMonitoring => {
                "Permite que o daemon verifique prazos e eventos periodicamente."
            }
            SettingId::PollInterval => "Intervalo entre verificações do daemon.",
            SettingId::NotificationsEnabled => {
                "Requer monitoramento em segundo plano e o daemon ativo; o backend é selecionado em tempo de execução."
            }
            SettingId::UpcomingTasks => {
                "Avisa uma vez quando a tarefa vence ou está agendada para o dia."
            }
            SettingId::OverdueTasks => "Avisa uma vez por dia sobre tarefas atrasadas.",
            SettingId::UpcomingEvents => {
                "Avisa uma vez no dia do evento e uma vez antes do horário de início."
            }
            SettingId::NotificationAdvance => {
                "Minutos antes do início de um evento em que o aviso deve ser enviado."
            }
        }
    }

    pub fn is_numeric(self) -> bool {
        matches!(
            self,
            SettingId::PollInterval | SettingId::NotificationAdvance
        )
    }

    pub fn value(self, config: &Config) -> String {
        match self {
            SettingId::StartWithSystem => bool_label(config.general.start_with_system),
            SettingId::BackgroundMonitoring => bool_label(config.general.background_monitoring),
            SettingId::PollInterval => {
                format!("{} segundos", config.general.poll_interval_secs)
            }
            SettingId::NotificationsEnabled => bool_label(config.notifications.enabled),
            SettingId::UpcomingTasks => bool_label(config.notifications.upcoming_tasks),
            SettingId::OverdueTasks => bool_label(config.notifications.overdue_tasks),
            SettingId::UpcomingEvents => bool_label(config.notifications.upcoming_events),
            SettingId::NotificationAdvance => {
                format!("{} minutos", config.notifications.advance_minutes)
            }
        }
    }

    pub fn bool_value(self, config: &Config) -> Option<bool> {
        match self {
            SettingId::StartWithSystem => Some(config.general.start_with_system),
            SettingId::BackgroundMonitoring => Some(config.general.background_monitoring),
            SettingId::NotificationsEnabled => Some(config.notifications.enabled),
            SettingId::UpcomingTasks => Some(config.notifications.upcoming_tasks),
            SettingId::OverdueTasks => Some(config.notifications.overdue_tasks),
            SettingId::UpcomingEvents => Some(config.notifications.upcoming_events),
            SettingId::PollInterval | SettingId::NotificationAdvance => None,
        }
    }

    pub fn toggle(self, config: &mut Config) {
        match self {
            SettingId::StartWithSystem => {
                config.general.start_with_system = !config.general.start_with_system;
            }
            SettingId::BackgroundMonitoring => {
                config.general.background_monitoring = !config.general.background_monitoring;
            }
            SettingId::NotificationsEnabled => {
                config.notifications.enabled = !config.notifications.enabled;
            }
            SettingId::UpcomingTasks => {
                config.notifications.upcoming_tasks = !config.notifications.upcoming_tasks;
            }
            SettingId::OverdueTasks => {
                config.notifications.overdue_tasks = !config.notifications.overdue_tasks;
            }
            SettingId::UpcomingEvents => {
                config.notifications.upcoming_events = !config.notifications.upcoming_events;
            }
            SettingId::PollInterval | SettingId::NotificationAdvance => {}
        }
    }

    pub fn numeric_value(self, config: &Config) -> Option<u64> {
        match self {
            SettingId::PollInterval => Some(config.general.poll_interval_secs),
            SettingId::NotificationAdvance => Some(config.notifications.advance_minutes as u64),
            _ => None,
        }
    }

    pub fn numeric_bounds(self) -> Option<(u64, u64)> {
        match self {
            SettingId::PollInterval => Some((MIN_POLL_INTERVAL_SECS, MAX_POLL_INTERVAL_SECS)),
            SettingId::NotificationAdvance => Some((
                MIN_NOTIFICATION_ADVANCE_MINUTES as u64,
                MAX_NOTIFICATION_ADVANCE_MINUTES as u64,
            )),
            _ => None,
        }
    }

    /// Aplica um valor numérico após validar os limites.
    ///
    /// Retorna uma mensagem amigável quando o valor está fora da faixa.
    pub fn set_numeric(self, config: &mut Config, value: u64) -> Result<(), String> {
        let Some((min, max)) = self.numeric_bounds() else {
            return Err("Esta configuração não é numérica.".to_string());
        };

        if value < min || value > max {
            return Err(format!("Valor inválido: use entre {} e {}.", min, max));
        }

        match self {
            SettingId::PollInterval => config.general.poll_interval_secs = value,
            SettingId::NotificationAdvance => config.notifications.advance_minutes = value as u32,
            _ => {}
        }
        Ok(())
    }
}

fn bool_label(value: bool) -> String {
    if value {
        "Ativado".to_string()
    } else {
        "Desativado".to_string()
    }
}

/// Estado de navegação e edição da tela de configurações.
#[derive(Debug, Clone)]
pub struct SettingsState {
    pub selected: usize,
    pub editing_value: Option<String>,
    pub feedback: Option<String>,
    pub return_view: View,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            selected: 0,
            editing_value: None,
            feedback: None,
            return_view: View::Calendario,
        }
    }
}

/// Listas de tarefas exibidas no dashboard, na mesma ordem da interface.
#[derive(Debug, Default)]
pub struct OverviewLists<'a> {
    pub overdue: Vec<&'a Task>,
    pub today: Vec<&'a Task>,
    pub upcoming: Vec<&'a Task>,
    pub no_date: Vec<&'a Task>,
}

/// Estado principal da aplicação Calma.
#[derive(Debug, Clone)]
pub struct App {
    /// Data de hoje obtida do sistema operacional.
    pub today: NaiveDate,
    /// Tarefas carregadas do SQLite.
    pub tasks: Vec<Task>,
    /// Projetos carregados do SQLite.
    pub projects: Vec<Project>,
    /// Eventos carregados do SQLite.
    pub events: Vec<Event>,
    /// Índice do item selecionado na visualização ativa de tarefas.
    pub selected_index: usize,
    /// Índice do item selecionado na visualização hierárquica.
    pub hierarchy_selected_idx: usize,
    /// Índice do evento selecionado na agenda.
    pub selected_event_index: usize,
    /// Índice da tarefa selecionada no painel lateral do calendário.
    pub calendar_selected_task_idx: usize,
    /// Índice da tarefa selecionada no dashboard.
    pub overview_selected_idx: usize,
    /// IDs de projetos atualmente expandidos na hierarquia.
    pub expanded_projects: HashSet<i64>,
    /// IDs de tarefas atualmente expandidas na hierarquia.
    pub expanded_tasks: HashSet<i64>,
    /// Se o modal de ajuda está visível.
    pub show_help: bool,
    /// Se a aplicação deve continuar rodando.
    pub running: bool,
    /// Visualização ativa.
    pub current_view: View,
    /// Modo de entrada atual.
    pub input_mode: InputMode,
    /// Buffer de texto digitado no modo de entrada rápida.
    pub input_buffer: String,
    /// Ano do calendário exibido.
    pub calendar_year: i32,
    /// Mês do calendário exibido (1-12).
    pub calendar_month: u32,
    /// Dia selecionado no calendário.
    pub selected_day: Option<NaiveDate>,
    /// Configuração centralizada carregada do disco.
    pub config: Config,
    /// Diagnóstico de ambiente gráfico exibido nas configurações.
    ///
    /// É derivado em tempo de execução e nunca persistido.
    pub desktop: DesktopDiagnostics,
    /// Estado de navegação/edição da tela de configurações.
    pub settings: SettingsState,
}

impl App {
    /// Cria uma nova instância da aplicação obtendo a data do sistema operacional.
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    /// Cria a aplicação usando uma configuração já carregada.
    pub fn with_config(config: Config) -> Self {
        let today = chrono::Local::now().date_naive();
        // A detecção do desktop é barata; a verificação do serviço D-Bus é
        // adiada até a abertura das configurações ou um pedido de refresh.
        let desktop = DesktopDiagnostics::detect_environment(&config);
        Self {
            today,
            tasks: Vec::new(),
            projects: Vec::new(),
            events: Vec::new(),
            selected_index: 0,
            hierarchy_selected_idx: 0,
            selected_event_index: 0,
            calendar_selected_task_idx: 0,
            overview_selected_idx: 0,
            expanded_projects: HashSet::new(),
            expanded_tasks: HashSet::new(),
            show_help: false,
            running: true,
            current_view: View::Calendario,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            calendar_year: today.year(),
            calendar_month: today.month(),
            selected_day: Some(today),
            config,
            desktop,
            settings: SettingsState::default(),
        }
    }

    /// Atualiza dinamicamente a data atual do sistema se a data do SO tiver mudado.
    pub fn update_current_date(&mut self) {
        let now = chrono::Local::now().date_naive();
        if self.today != now {
            self.today = now;
        }
    }

    /// Recarrega tarefas, projetos e eventos do SQLite mantendo seleções válidas.
    pub fn reload(&mut self, conn: &Connection) -> AppResult<()> {
        self.tasks = crate::db::list_tasks(conn)?;
        self.projects = crate::db::list_projects(conn)?;
        self.events = crate::db::list_events(conn)?;

        // Ajusta selected_index da lista plana de tarefas
        if self.tasks.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.tasks.len() {
            self.selected_index = self.tasks.len() - 1;
        }

        // Ajusta hierarchy_selected_idx
        let items_len = self.build_hierarchy_items().len();
        if items_len == 0 {
            self.hierarchy_selected_idx = 0;
        } else if self.hierarchy_selected_idx >= items_len {
            self.hierarchy_selected_idx = items_len - 1;
        }

        // Ajusta a seleção da agenda de eventos
        if self.events.is_empty() {
            self.selected_event_index = 0;
        } else if self.selected_event_index >= self.events.len() {
            self.selected_event_index = self.events.len() - 1;
        }

        // Ajusta a seleção do painel lateral do calendário
        let day = self.selected_day.unwrap_or(self.today);
        let day_tasks_len = self.tasks_for_day(day).len();
        if day_tasks_len == 0 {
            self.calendar_selected_task_idx = 0;
        } else if self.calendar_selected_task_idx >= day_tasks_len {
            self.calendar_selected_task_idx = day_tasks_len - 1;
        }

        // Ajusta a seleção do dashboard
        let overview_len = self.overview_flat_len();
        if overview_len == 0 {
            self.overview_selected_idx = 0;
        } else if self.overview_selected_idx >= overview_len {
            self.overview_selected_idx = overview_len - 1;
        }

        Ok(())
    }

    // ==========================================
    // SELEÇÃO CONTEXTUAL (CALENDÁRIO, DASHBOARD E EVENTOS)
    // ==========================================

    /// Retorna as tarefas com vencimento ou agendamento no dia informado.
    pub fn tasks_for_day(&self, day: NaiveDate) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.due_date == Some(day) || t.scheduled_date == Some(day))
            .collect()
    }

    /// Retorna a tarefa selecionada no painel lateral do calendário.
    pub fn selected_calendar_task(&self) -> Option<&Task> {
        let day = self.selected_day.unwrap_or(self.today);
        self.tasks_for_day(day)
            .get(self.calendar_selected_task_idx)
            .copied()
    }

    /// Monta as listas do dashboard na mesma ordem exibida na tela.
    pub fn overview_lists(&self) -> OverviewLists<'_> {
        let mut overdue: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| {
                t.status != TaskStatus::Concluida && t.due_date.is_some_and(|d| d < self.today)
            })
            .collect();
        overdue.sort_by_key(|t| t.due_date);

        let mut today: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.status != TaskStatus::Concluida && t.due_date == Some(self.today))
            .collect();
        today.sort_by_key(|a| a.title.to_lowercase());

        let next_week = self.today + Duration::days(7);
        let mut upcoming: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| {
                t.status != TaskStatus::Concluida
                    && t.due_date.is_some_and(|d| d > self.today && d <= next_week)
            })
            .collect();
        upcoming.sort_by_key(|t| t.due_date);

        let mut no_date: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.status != TaskStatus::Concluida && t.due_date.is_none())
            .collect();
        no_date.sort_by_key(|a| a.title.to_lowercase());

        OverviewLists {
            overdue,
            today,
            upcoming,
            no_date,
        }
    }

    /// Quantidade total de tarefas navegáveis no dashboard.
    pub fn overview_flat_len(&self) -> usize {
        let lists = self.overview_lists();
        lists.overdue.len() + lists.today.len() + lists.upcoming.len() + lists.no_date.len()
    }

    /// Retorna a tarefa selecionada no dashboard.
    pub fn selected_overview_task(&self) -> Option<&Task> {
        let lists = self.overview_lists();
        let mut idx = self.overview_selected_idx;

        if idx < lists.overdue.len() {
            return Some(lists.overdue[idx]);
        }
        idx -= lists.overdue.len();

        if idx < lists.today.len() {
            return Some(lists.today[idx]);
        }
        idx -= lists.today.len();

        if idx < lists.upcoming.len() {
            return Some(lists.upcoming[idx]);
        }
        idx -= lists.upcoming.len();

        lists.no_date.get(idx).copied()
    }

    /// Retorna o evento atualmente selecionado na agenda.
    pub fn selected_event(&self) -> Option<&Event> {
        self.events.get(self.selected_event_index)
    }

    /// Retorna a tarefa selecionada de acordo com a visualização ativa.
    pub fn selected_task(&self) -> Option<&Task> {
        match self.current_view {
            View::Tarefas => self.tasks.get(self.selected_index),
            View::Hierarquia => match self.get_selected_hierarchy_item()?.id {
                HierarchyNodeId::Task(id) => self.tasks.iter().find(|t| t.id == id),
                HierarchyNodeId::Project(_) => None,
            },
            View::Calendario => self.selected_calendar_task(),
            View::VisaoGeral => self.selected_overview_task(),
            View::Eventos | View::Configuracoes => None,
        }
    }

    // ==========================================
    // HIERARQUIA RECURSIVA & EXPANSÃO/RECOLHIMENTO
    // ==========================================

    /// Constrói a lista plana de nós visíveis na árvore hierárquica respeitando a expansão.
    pub fn build_hierarchy_items(&self) -> Vec<HierarchyItem> {
        let mut items = Vec::new();

        // 1. Projetos raiz (parent_id == None)
        let root_projects: Vec<&Project> = self
            .projects
            .iter()
            .filter(|p| p.parent_id.is_none())
            .collect();

        for proj in root_projects {
            self.traverse_project(proj, 0, &mut items);
        }

        // 2. Tarefas sem projeto e sem pai
        let unassigned_root_tasks: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.project_id.is_none() && t.parent_id.is_none())
            .collect();

        if !unassigned_root_tasks.is_empty() {
            for task in unassigned_root_tasks {
                self.traverse_task(task, 0, &mut items);
            }
        }

        items
    }

    fn traverse_project(&self, proj: &Project, depth: usize, items: &mut Vec<HierarchyItem>) {
        let subprojects: Vec<&Project> = self
            .projects
            .iter()
            .filter(|p| p.parent_id == Some(proj.id))
            .collect();

        let proj_root_tasks: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.project_id == Some(proj.id) && t.parent_id.is_none())
            .collect();

        let has_children = !subprojects.is_empty() || !proj_root_tasks.is_empty();
        let is_expanded = self.expanded_projects.contains(&proj.id);
        let progress = self.compute_project_progress(proj.id);

        items.push(HierarchyItem {
            id: HierarchyNodeId::Project(proj.id),
            title: proj.name.clone(),
            depth,
            is_project: true,
            has_children,
            is_expanded,
            status: None,
            due_date: None,
            priority: None,
            progress: Some(progress),
        });

        if is_expanded {
            // Subprojetos recursivos
            for sub in subprojects {
                self.traverse_project(sub, depth + 1, items);
            }
            // Tarefas raiz do projeto
            for task in proj_root_tasks {
                self.traverse_task(task, depth + 1, items);
            }
        }
    }

    fn traverse_task(&self, task: &Task, depth: usize, items: &mut Vec<HierarchyItem>) {
        let subtasks: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.parent_id == Some(task.id))
            .collect();

        let has_children = !subtasks.is_empty();
        let is_expanded = self.expanded_tasks.contains(&task.id);
        let progress = if has_children {
            Some(self.compute_task_progress(task.id))
        } else {
            None
        };

        items.push(HierarchyItem {
            id: HierarchyNodeId::Task(task.id),
            title: task.title.clone(),
            depth,
            is_project: false,
            has_children,
            is_expanded,
            status: Some(task.status),
            due_date: task.due_date,
            priority: Some(task.priority),
            progress,
        });

        if is_expanded {
            for sub in subtasks {
                self.traverse_task(sub, depth + 1, items);
            }
        }
    }

    /// Calcula recursivamente o progresso de um projeto (total de tarefas concluídas vs total de tarefas).
    pub fn compute_project_progress(&self, proj_id: i64) -> (usize, usize) {
        let mut all_project_ids = HashSet::new();
        let mut queue = vec![proj_id];
        while let Some(pid) = queue.pop() {
            if all_project_ids.insert(pid) {
                for p in &self.projects {
                    if p.parent_id == Some(pid) {
                        queue.push(p.id);
                    }
                }
            }
        }

        let tasks_in_projects: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| {
                t.project_id
                    .is_some_and(|pid| all_project_ids.contains(&pid))
            })
            .collect();

        let total = tasks_in_projects.len();
        let completed = tasks_in_projects
            .iter()
            .filter(|t| t.status == TaskStatus::Concluida)
            .count();
        (completed, total)
    }

    /// Calcula recursivamente o progresso de uma tarefa com subtarefas.
    pub fn compute_task_progress(&self, task_id: i64) -> (usize, usize) {
        let mut all_subtask_ids = HashSet::new();
        let mut queue = vec![task_id];
        while let Some(tid) = queue.pop() {
            for t in &self.tasks {
                if t.parent_id == Some(tid) && all_subtask_ids.insert(t.id) {
                    queue.push(t.id);
                }
            }
        }

        let total = all_subtask_ids.len();
        let completed = self
            .tasks
            .iter()
            .filter(|t| all_subtask_ids.contains(&t.id) && t.status == TaskStatus::Concluida)
            .count();
        (completed, total)
    }

    /// Alterna expansão/recolhimento do nó selecionado na hierarquia.
    pub fn toggle_hierarchy_expand(&mut self) {
        let items = self.build_hierarchy_items();
        if let Some(item) = items.get(self.hierarchy_selected_idx) {
            match item.id {
                HierarchyNodeId::Project(pid) => {
                    if self.expanded_projects.contains(&pid) {
                        self.expanded_projects.remove(&pid);
                    } else {
                        self.expanded_projects.insert(pid);
                    }
                }
                HierarchyNodeId::Task(tid) => {
                    if self.expanded_tasks.contains(&tid) {
                        self.expanded_tasks.remove(&tid);
                    } else {
                        self.expanded_tasks.insert(tid);
                    }
                }
            }
        }
    }

    /// Retorna o item atualmente selecionado na hierarquia.
    pub fn get_selected_hierarchy_item(&self) -> Option<HierarchyItem> {
        let items = self.build_hierarchy_items();
        items.get(self.hierarchy_selected_idx).cloned()
    }

    // ==========================================
    // EXCLUSÃO COM CONFIRMAÇÃO (Prioridade 2)
    // ==========================================

    /// Inicia o diálogo de confirmação de exclusão para o item selecionado no contexto atual.
    pub fn start_confirm_delete(&mut self) {
        match self.current_view {
            View::Eventos => {
                if let Some((id, title)) = self.selected_event().map(|e| (e.id, e.title.clone())) {
                    self.input_mode = InputMode::ConfirmDelete {
                        target: DeleteTarget::Event,
                        id,
                        title,
                    };
                }
            }
            View::Hierarquia => {
                if let Some(item) = self.get_selected_hierarchy_item() {
                    let (target, id) = match item.id {
                        HierarchyNodeId::Task(id) => (DeleteTarget::Task, id),
                        HierarchyNodeId::Project(id) => (DeleteTarget::Project, id),
                    };
                    self.input_mode = InputMode::ConfirmDelete {
                        target,
                        id,
                        title: item.title,
                    };
                }
            }
            View::Tarefas | View::Calendario | View::VisaoGeral => {
                if let Some((id, title)) = self.selected_task().map(|t| (t.id, t.title.clone())) {
                    self.input_mode = InputMode::ConfirmDelete {
                        target: DeleteTarget::Task,
                        id,
                        title,
                    };
                }
            }
            View::Configuracoes => {}
        }
    }

    /// Confirma e executa a exclusão após aprovação do usuário.
    pub fn confirm_delete(&mut self, conn: &Connection) -> AppResult<()> {
        if let InputMode::ConfirmDelete { target, id, .. } = self.input_mode {
            match target {
                DeleteTarget::Project => {
                    crate::db::delete_project(conn, id)?;
                    self.expanded_projects.remove(&id);
                }
                DeleteTarget::Task => {
                    crate::db::delete_task(conn, id)?;
                    self.expanded_tasks.remove(&id);
                }
                DeleteTarget::Event => {
                    crate::db::delete_event(conn, id)?;
                }
            }
            self.input_mode = InputMode::Normal;
            self.reload(conn)?;
        }
        Ok(())
    }

    // ==========================================
    // EDIÇÃO DE TAREFA E PROJETO (Prioridade 3 & 4)
    // ==========================================

    /// Inicia o formulário de edição para o item selecionado.
    pub fn start_edit(&mut self) {
        match self.current_view {
            View::Eventos => {
                if let Some(event) = self.selected_event() {
                    let state = EventEditState {
                        event_id: event.id,
                        active_field: 0,
                        title: event.title.clone(),
                        date_str: event.date.format("%d/%m/%Y").to_string(),
                        time_str: event
                            .start_time
                            .map(|t| t.format("%H:%M").to_string())
                            .unwrap_or_default(),
                    };
                    self.input_mode = InputMode::EditEvent(state);
                }
            }
            View::Hierarquia => {
                if let Some(item) = self.get_selected_hierarchy_item() {
                    match item.id {
                        HierarchyNodeId::Task(id) => {
                            let state = self
                                .tasks
                                .iter()
                                .find(|t| t.id == id)
                                .map(task_to_edit_state);
                            if let Some(state) = state {
                                self.input_mode = InputMode::EditTask(state);
                            }
                        }
                        HierarchyNodeId::Project(id) => {
                            let project = self
                                .projects
                                .iter()
                                .find(|p| p.id == id)
                                .map(|p| (p.id, p.name.clone()));
                            if let Some((project_id, buffer)) = project {
                                self.input_mode = InputMode::EditProject { project_id, buffer };
                            }
                        }
                    }
                }
            }
            View::Tarefas | View::Calendario | View::VisaoGeral => {
                let state = self.selected_task().map(task_to_edit_state);
                if let Some(state) = state {
                    self.input_mode = InputMode::EditTask(state);
                }
            }
            View::Configuracoes => {}
        }
    }

    /// Salva as alterações da tarefa em edição.
    pub fn save_edit_task(&mut self, conn: &Connection) -> AppResult<()> {
        if let InputMode::EditTask(edit) = &self.input_mode {
            let edit = edit.clone();
            if let Some(mut task) = self.tasks.iter().find(|t| t.id == edit.task_id).cloned() {
                task.title = edit.title.trim().to_string();
                if !edit.due_date_str.trim().is_empty() {
                    task.due_date = parse_pt_date(&edit.due_date_str, self.today);
                } else {
                    task.due_date = None;
                }
                task.priority = edit.priority;
                task.project_id = edit.project_id;

                crate::db::update_task(conn, &task)?;
                self.input_mode = InputMode::Normal;
                self.reload(conn)?;
            }
        }
        Ok(())
    }

    /// Salva as alterações do evento em edição.
    pub fn save_edit_event(&mut self, conn: &Connection) -> AppResult<()> {
        if let InputMode::EditEvent(edit) = &self.input_mode {
            let edit = edit.clone();
            if let Some(mut event) = self.events.iter().find(|e| e.id == edit.event_id).cloned() {
                let title = edit.title.trim();
                if !title.is_empty() {
                    event.title = title.to_string();
                }

                if let Some(date) = parse_pt_date(&edit.date_str, self.today) {
                    event.date = date;
                }

                let time_str = edit.time_str.trim();
                if time_str.is_empty() {
                    event.start_time = None;
                } else if let Some(time) = parse_time(time_str) {
                    event.start_time = Some(time);
                }

                crate::db::update_event(conn, &event)?;
                self.input_mode = InputMode::Normal;
                self.reload(conn)?;
            }
        }
        Ok(())
    }

    /// Salva a renomeação de um projeto.
    pub fn save_edit_project(&mut self, conn: &Connection) -> AppResult<()> {
        if let InputMode::EditProject { project_id, buffer } = &self.input_mode {
            let pid = *project_id;
            let name = buffer.trim().to_string();
            if !name.is_empty() {
                if let Some(mut proj) = self.projects.iter().find(|p| p.id == pid).cloned() {
                    proj.name = name;
                    crate::db::update_project(conn, &proj)?;
                }
            }
            self.input_mode = InputMode::Normal;
            self.reload(conn)?;
        }
        Ok(())
    }

    /// Inicia o diálogo de criação de uma subtarefa para o item atual.
    pub fn start_new_subtask(&mut self) {
        if let Some(item) = self.get_selected_hierarchy_item() {
            if let HierarchyNodeId::Task(parent_id) = item.id {
                self.input_mode = InputMode::NewSubtask {
                    parent_task_id: parent_id,
                    parent_task_title: item.title,
                    buffer: String::new(),
                };
            }
        }
    }

    /// Salva uma nova subtarefa diretamente conectada à tarefa pai.
    pub fn save_new_subtask(&mut self, conn: &Connection) -> AppResult<()> {
        if let InputMode::NewSubtask {
            parent_task_id,
            buffer,
            ..
        } = &self.input_mode
        {
            let pid = *parent_task_id;
            let input = buffer.trim();
            if !input.is_empty() {
                let parsed = parse_quick_task(input, self.today);
                let parent_proj_id = self
                    .tasks
                    .iter()
                    .find(|t| t.id == pid)
                    .and_then(|t| t.project_id);

                let mut task = Task::new(parsed.title);
                task.parent_id = Some(pid);
                task.project_id = parent_proj_id;
                task.due_date = parsed.due_date;

                crate::db::insert_task(conn, &task)?;
                self.expanded_tasks.insert(pid);
            }
            self.input_mode = InputMode::Normal;
            self.reload(conn)?;
        }
        Ok(())
    }

    /// Inicia o diálogo de criação de projeto ou subprojeto.
    pub fn start_new_project(&mut self) {
        let parent_project_id = if self.current_view == View::Hierarquia {
            self.get_selected_hierarchy_item().and_then(|item| {
                if let HierarchyNodeId::Project(pid) = item.id {
                    Some(pid)
                } else {
                    None
                }
            })
        } else {
            None
        };

        self.input_mode = InputMode::NewProject {
            parent_project_id,
            buffer: String::new(),
        };
    }

    /// Salva um novo projeto ou subprojeto.
    pub fn save_new_project(&mut self, conn: &Connection) -> AppResult<()> {
        if let InputMode::NewProject {
            parent_project_id,
            buffer,
        } = &self.input_mode
        {
            let name = buffer.trim().to_string();
            let parent_id = *parent_project_id;
            if !name.is_empty() {
                let mut proj = Project::new(name);
                proj.parent_id = parent_id;
                let new_id = crate::db::insert_project(conn, &proj)?;

                if let Some(pid) = parent_id {
                    self.expanded_projects.insert(pid);
                }
                self.expanded_projects.insert(new_id);
            }
            self.input_mode = InputMode::Normal;
            self.reload(conn)?;
        }
        Ok(())
    }

    // ==========================================
    // OPERAÇÕES GERAIS DE TAREFAS
    // ==========================================

    /// Adiciona uma tarefa a partir do buffer digitado pelo usuário (interpretação PT-BR).
    pub fn add_task_from_buffer(&mut self, conn: &Connection) -> AppResult<()> {
        let input = self.input_buffer.trim();
        if input.is_empty() {
            self.input_mode = InputMode::Normal;
            self.input_buffer.clear();
            return Ok(());
        }

        let parsed = parse_quick_task(input, self.today);
        let mut task = Task::new(parsed.title);
        task.due_date = parsed.due_date;

        // Se estivermos em um projeto na hierarquia, vincula a ele
        if self.current_view == View::Hierarquia {
            if let Some(item) = self.get_selected_hierarchy_item() {
                if let HierarchyNodeId::Project(pid) = item.id {
                    task.project_id = Some(pid);
                    self.expanded_projects.insert(pid);
                }
            }
        }

        crate::db::insert_task(conn, &task)?;
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();

        self.reload(conn)?;
        Ok(())
    }

    /// Abre o cadastro rápido de um novo evento.
    pub fn start_new_event(&mut self) {
        self.input_mode = InputMode::QuickAddEvent;
        self.input_buffer.clear();
    }

    /// Adiciona um evento a partir do buffer digitado pelo usuário.
    ///
    /// Aceita o prefixo opcional `event:` (usado no cadastro rápido global) e
    /// interpreta data e horário em linguagem natural.
    pub fn add_event_from_buffer(&mut self, conn: &Connection) -> AppResult<()> {
        let raw = self.input_buffer.trim().to_string();
        let input = if raw.to_lowercase().starts_with("event:") {
            raw[6..].trim().to_string()
        } else {
            raw
        };

        if input.is_empty() {
            self.input_mode = InputMode::Normal;
            self.input_buffer.clear();
            return Ok(());
        }

        let parsed = parse_quick_event(&input, self.today);
        let title = if parsed.title.trim().is_empty() {
            "Evento".to_string()
        } else {
            parsed.title
        };
        let mut event = Event::new(title, parsed.date);
        event.start_time = parsed.start_time;
        event.end_time = parsed.end_time;
        event.description = Some(input);

        crate::db::insert_event(conn, &event)?;
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();

        self.reload(conn)?;
        Ok(())
    }

    /// Alterna a conclusão da tarefa atualmente selecionada.
    pub fn toggle_selected_task(&mut self, conn: &Connection) -> AppResult<()> {
        if let Some(task_id) = self.selected_task().map(|t| t.id) {
            let is_completed = self
                .tasks
                .iter()
                .find(|t| t.id == task_id)
                .is_some_and(|t| t.status == TaskStatus::Concluida);
            crate::db::mark_task_completed(conn, task_id, !is_completed)?;
            self.reload(conn)?;
        }
        Ok(())
    }

    /// Volta para a data de hoje no calendário.
    pub fn go_to_today(&mut self) {
        self.today = chrono::Local::now().date_naive();
        self.calendar_year = self.today.year();
        self.calendar_month = self.today.month();
        self.selected_day = Some(self.today);
        self.calendar_selected_task_idx = 0;
    }

    /// Seleciona o item anterior na lista ativa.
    pub fn select_previous(&mut self) {
        match self.current_view {
            View::Hierarquia => {
                if self.hierarchy_selected_idx > 0 {
                    self.hierarchy_selected_idx -= 1;
                }
            }
            View::Eventos => {
                if self.selected_event_index > 0 {
                    self.selected_event_index -= 1;
                }
            }
            View::Calendario => {
                if self.calendar_selected_task_idx > 0 {
                    self.calendar_selected_task_idx -= 1;
                }
            }
            View::VisaoGeral => {
                if self.overview_selected_idx > 0 {
                    self.overview_selected_idx -= 1;
                }
            }
            View::Tarefas => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            View::Configuracoes => {}
        }
    }

    /// Seleciona o próximo item na lista ativa.
    pub fn select_next(&mut self) {
        match self.current_view {
            View::Hierarquia => {
                let max = self.build_hierarchy_items().len().saturating_sub(1);
                self.hierarchy_selected_idx = (self.hierarchy_selected_idx + 1).min(max);
            }
            View::Eventos => {
                let max = self.events.len().saturating_sub(1);
                self.selected_event_index = (self.selected_event_index + 1).min(max);
            }
            View::Calendario => {
                let day = self.selected_day.unwrap_or(self.today);
                let max = self.tasks_for_day(day).len().saturating_sub(1);
                self.calendar_selected_task_idx = (self.calendar_selected_task_idx + 1).min(max);
            }
            View::VisaoGeral => {
                let max = self.overview_flat_len().saturating_sub(1);
                self.overview_selected_idx = (self.overview_selected_idx + 1).min(max);
            }
            View::Tarefas => {
                let max = self.tasks.len().saturating_sub(1);
                self.selected_index = (self.selected_index + 1).min(max);
            }
            View::Configuracoes => {}
        }
    }

    /// Avança para o próximo mês no calendário.
    pub fn next_month(&mut self) {
        if self.calendar_month == 12 {
            self.calendar_month = 1;
            self.calendar_year += 1;
        } else {
            self.calendar_month += 1;
        }
        self.selected_day = None;
        self.calendar_selected_task_idx = 0;
    }

    /// Retorna para o mês anterior no calendário.
    pub fn previous_month(&mut self) {
        if self.calendar_month == 1 {
            self.calendar_month = 12;
            self.calendar_year -= 1;
        } else {
            self.calendar_month -= 1;
        }
        self.selected_day = None;
        self.calendar_selected_task_idx = 0;
    }

    /// Seleciona um dia no calendário.
    pub fn select_day(&mut self, day: NaiveDate) {
        self.selected_day = Some(day);
        self.calendar_selected_task_idx = 0;
    }

    // ==========================================
    // CONFIGURAÇÕES PERSISTENTES
    // ==========================================

    /// Abre a tela de configurações guardando a visualização anterior.
    pub fn open_settings(&mut self) {
        if self.current_view != View::Configuracoes {
            self.settings.return_view = self.current_view;
            self.current_view = View::Configuracoes;
            self.settings.feedback = None;
            self.settings.editing_value = None;
            self.desktop = DesktopDiagnostics::detect(&self.config);
        }
    }

    /// Reexecuta a detecção do ambiente e a verificação do serviço.
    ///
    /// Não altera configurações e não persiste nenhum resultado.
    pub fn refresh_desktop_diagnostics(&mut self) {
        self.desktop = DesktopDiagnostics::detect(&self.config);
        self.settings.feedback = Some("Diagnóstico de ambiente atualizado.".to_string());
    }

    /// Retorna à visualização anterior sem descartar configurações salvas.
    pub fn close_settings(&mut self) {
        self.settings.editing_value = None;
        self.settings.feedback = None;
        self.current_view = self.settings.return_view;
    }

    /// Configuração atualmente selecionada.
    pub fn selected_setting(&self) -> SettingId {
        let max = SettingId::ALL.len().saturating_sub(1);
        SettingId::ALL[self.settings.selected.min(max)]
    }

    /// Move a seleção para a configuração anterior.
    pub fn settings_select_previous(&mut self) {
        if self.settings.selected > 0 {
            self.settings.selected -= 1;
            self.settings.feedback = None;
        }
    }

    /// Move a seleção para a próxima configuração.
    pub fn settings_select_next(&mut self) {
        let max = SettingId::ALL.len().saturating_sub(1);
        if self.settings.selected < max {
            self.settings.selected += 1;
            self.settings.feedback = None;
        }
    }

    /// Ativa a configuração selecionada.
    ///
    /// Booleanos são alternados e salvos imediatamente. Valores numéricos
    /// entram em modo de edição.
    pub fn settings_activate_selected(&mut self) {
        self.settings.feedback = None;
        let setting = self.selected_setting();

        if setting.is_numeric() {
            if let Some(value) = setting.numeric_value(&self.config) {
                self.settings.editing_value = Some(value.to_string());
            }
            return;
        }

        setting.toggle(&mut self.config);
        self.persist_settings();
    }

    /// Cancela a edição de um valor numérico.
    pub fn settings_cancel_edit(&mut self) {
        self.settings.editing_value = None;
        self.settings.feedback = None;
    }

    /// Confirma a edição do valor numérico selecionado.
    ///
    /// Em caso de erro, a edição permanece aberta para correção e o motivo é
    /// exibido como feedback na tela.
    pub fn settings_commit_edit(&mut self) {
        let Some(buffer) = self.settings.editing_value.clone() else {
            return;
        };

        let setting = self.selected_setting();
        let parsed = buffer.trim().parse::<u64>();
        let result = match parsed {
            Ok(value) => setting.set_numeric(&mut self.config, value),
            Err(_) => Err("Digite um número inteiro válido.".to_string()),
        };

        match result {
            Ok(()) => {
                self.settings.editing_value = None;
                self.persist_settings();
            }
            Err(message) => {
                self.settings.feedback = Some(message);
            }
        }
    }

    /// Acrescenta um caractere à edição numérica atual.
    pub fn settings_push_char(&mut self, c: char) {
        if let Some(buffer) = self.settings.editing_value.as_mut() {
            if c.is_ascii_digit() {
                buffer.push(c);
                self.settings.feedback = None;
            }
        }
    }

    /// Remove o último caractere da edição numérica atual.
    pub fn settings_pop_char(&mut self) {
        if let Some(buffer) = self.settings.editing_value.as_mut() {
            buffer.pop();
            self.settings.feedback = None;
        }
    }

    /// Persiste a configuração e produz feedback para o usuário.
    ///
    /// Falhas de escrita não encerram a aplicação: o erro é mostrado na
    /// própria tela de configurações.
    fn persist_settings(&mut self) {
        match self.config.save() {
            Ok(()) => {
                self.config.load_warning = None;
                self.settings.feedback = Some(format!(
                    "Configurações salvas em {}",
                    self.config.config_path.display()
                ));
            }
            Err(err) => {
                self.settings.feedback = Some(format!("Não foi possível salvar: {}", err));
            }
        }
    }

    /// Encerra a aplicação.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

/// Converte uma tarefa no estado inicial do formulário de edição.
fn task_to_edit_state(task: &Task) -> TaskEditState {
    TaskEditState {
        task_id: task.id,
        active_field: 0,
        title: task.title.clone(),
        due_date_str: task
            .due_date
            .map(|d| d.format("%d/%m/%Y").to_string())
            .unwrap_or_default(),
        priority: task.priority,
        project_id: task.project_id,
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_hierarchy_faculdade_calculo_lista3_questao1() {
        let mut app = App::new();

        // 1. Projeto Raiz: Faculdade
        let mut proj_faculdade = Project::new("Faculdade".into());
        proj_faculdade.id = 1;
        app.projects.push(proj_faculdade);

        // 2. Subprojeto: Cálculo I (parent_id = 1)
        let mut proj_calculo = Project::new("Cálculo I".into());
        proj_calculo.id = 2;
        proj_calculo.parent_id = Some(1);
        app.projects.push(proj_calculo);

        // 3. Tarefa: Lista 3 (project_id = 2, parent_id = None)
        let mut task_lista3 = Task::new("Lista 3".into());
        task_lista3.id = 10;
        task_lista3.project_id = Some(2);
        app.tasks.push(task_lista3);

        // 4. Subtarefa: Questão 1 (project_id = 2, parent_id = 10)
        let mut task_q1 = Task::new("Questão 1".into());
        task_q1.id = 11;
        task_q1.project_id = Some(2);
        task_q1.parent_id = Some(10);
        app.tasks.push(task_q1);

        // Estado inicial: tudo recolhido
        let items_collapsed = app.build_hierarchy_items();
        assert_eq!(items_collapsed.len(), 1);
        assert_eq!(items_collapsed[0].title, "Faculdade");
        assert_eq!(items_collapsed[0].depth, 0);

        // Expande Faculdade
        app.expanded_projects.insert(1);
        let items_step1 = app.build_hierarchy_items();
        assert_eq!(items_step1.len(), 2);
        assert_eq!(items_step1[0].title, "Faculdade");
        assert_eq!(items_step1[1].title, "Cálculo I");
        assert_eq!(items_step1[1].depth, 1);

        // Expande Cálculo I
        app.expanded_projects.insert(2);
        let items_step2 = app.build_hierarchy_items();
        assert_eq!(items_step2.len(), 3);
        assert_eq!(items_step2[2].title, "Lista 3");
        assert_eq!(items_step2[2].depth, 2);

        // Expande Lista 3
        app.expanded_tasks.insert(10);
        let items_step3 = app.build_hierarchy_items();
        assert_eq!(items_step3.len(), 4);
        assert_eq!(items_step3[3].title, "Questão 1");
        assert_eq!(items_step3[3].depth, 3);
    }

    #[test]
    fn test_recursive_progress_calculation() {
        let mut app = App::new();

        let mut proj = Project::new("Engenharia".into());
        proj.id = 1;
        app.projects.push(proj);

        let mut t1 = Task::new("T1".into());
        t1.id = 1;
        t1.project_id = Some(1);
        t1.status = TaskStatus::Concluida;
        app.tasks.push(t1);

        let mut t2 = Task::new("T2".into());
        t2.id = 2;
        t2.project_id = Some(1);
        t2.status = TaskStatus::Pendente;
        app.tasks.push(t2);

        let (completed, total) = app.compute_project_progress(1);
        assert_eq!(completed, 1);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_overview_lists_and_context_selection() {
        let mut app = App::new();
        let today = app.today;

        let mut overdue = Task::new("Atrasada".into());
        overdue.id = 1;
        overdue.due_date = Some(today - Duration::days(2));
        app.tasks.push(overdue);

        let mut today_task = Task::new("Hoje".into());
        today_task.id = 2;
        today_task.due_date = Some(today);
        app.tasks.push(today_task);

        let mut upcoming = Task::new("Próxima".into());
        upcoming.id = 3;
        upcoming.due_date = Some(today + Duration::days(3));
        app.tasks.push(upcoming);

        let mut no_date = Task::new("Sem data".into());
        no_date.id = 4;
        app.tasks.push(no_date);

        let lists = app.overview_lists();
        assert_eq!(lists.overdue.len(), 1);
        assert_eq!(lists.today.len(), 1);
        assert_eq!(lists.upcoming.len(), 1);
        assert_eq!(lists.no_date.len(), 1);
        assert_eq!(app.overview_flat_len(), 4);

        app.current_view = View::VisaoGeral;
        app.overview_selected_idx = 0;
        assert_eq!(app.selected_task().map(|t| t.id), Some(1));

        app.select_next();
        assert_eq!(app.selected_task().map(|t| t.id), Some(2));

        app.select_next();
        app.select_next();
        assert_eq!(app.selected_task().map(|t| t.id), Some(4));

        // No calendário, a seleção vem do painel lateral do dia escolhido.
        app.current_view = View::Calendario;
        app.selected_day = Some(today);
        app.calendar_selected_task_idx = 0;
        assert_eq!(app.selected_task().map(|t| t.id), Some(2));
    }

    #[test]
    fn test_settings_toggle_persists_immediately() {
        let path = temp_config_path("app-toggle");
        let config = Config {
            config_path: path.clone(),
            ..Config::default()
        };
        let mut app = App::with_config(config);

        app.open_settings();
        assert_eq!(app.current_view, View::Configuracoes);
        assert_eq!(app.selected_setting(), SettingId::StartWithSystem);

        app.settings_activate_selected();
        assert!(app.config.general.start_with_system);
        assert!(app.settings.feedback.is_some());

        let reloaded = Config::load_from(&path);
        assert!(reloaded.general.start_with_system);

        app.close_settings();
        assert_eq!(app.current_view, View::Calendario);

        cleanup_config(&path);
    }

    #[test]
    fn desktop_diagnostics_reflect_runtime_configuration() {
        let mut app = App::new();
        app.config.notifications.enabled = true;

        app.refresh_desktop_diagnostics();

        assert!(app.desktop.delivery_enabled);
        assert!(
            app.settings
                .feedback
                .as_deref()
                .is_some_and(|message| message.contains("Diagnóstico"))
        );
        // O resultado da detecção nunca é gravado na configuração.
        assert_eq!(
            app.config.notifications.desktop.backend,
            crate::config::DEFAULT_DESKTOP_NOTIFICATION_BACKEND
        );
    }

    #[test]
    fn test_numeric_setting_validates_and_persists() {
        let path = temp_config_path("app-numeric");
        let config = Config {
            config_path: path.clone(),
            ..Config::default()
        };
        let mut app = App::with_config(config);

        app.settings.selected = 2; // PollInterval
        assert_eq!(app.selected_setting(), SettingId::PollInterval);
        app.settings_activate_selected();
        assert_eq!(
            app.settings.editing_value,
            Some(crate::config::DEFAULT_POLL_INTERVAL_SECS.to_string())
        );

        // Valor inválido mantém a edição aberta e informa o usuário.
        app.settings.editing_value = Some("0".to_string());
        app.settings_commit_edit();
        assert!(app.settings.editing_value.is_some());
        assert!(app.settings.feedback.is_some());

        // Valor válido é aplicado e salvo.
        app.settings.editing_value = Some("120".to_string());
        app.settings_commit_edit();
        assert!(app.settings.editing_value.is_none());
        assert_eq!(app.config.general.poll_interval_secs, 120);

        let reloaded = Config::load_from(&path);
        assert_eq!(reloaded.general.poll_interval_secs, 120);

        cleanup_config(&path);
    }

    fn temp_config_path(name: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("relógio do sistema")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "calma-app-test-{}-{}-{}",
                std::process::id(),
                nanos,
                name
            ))
            .join(crate::config::CONFIG_FILE_NAME)
    }

    fn cleanup_config(path: &std::path::Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }
}
