//! Ícone de bandeja do sistema.
//!
//! A implementação inicial usa o StatusNotifierItem, o protocolo nativo do
//! KDE Plasma (e também suportado por outros ambientes por meio de
//! extensões). O desenho é deliberadamente abstrato:
//!
//! - [`TrayState`] e [`TraySummary`] descrevem o que deve ser exibido sem
//!   conhecer D-Bus;
//! - [`TrayBackend`] permite registrar novos backends no futuro (GNOME,
//!   XFCE, etc.) sem alterar o daemon;
//! - [`TrayHandle`] permite atualizar o ícone e encerrar o serviço de forma
//!   independente do backend concreto.
//!
//! O clique no ícone abre a TUI por meio de [`crate::desktop::terminal`],
//! usando o terminal padrão detectado no sistema.

pub mod icon;
mod kde;

use crate::config::Config;
use crate::desktop::detection::DesktopSession;
use crate::desktop::terminal::{TerminalLauncher, TuiLauncher};
use crate::errors::{AppError, AppResult};
use crate::models::task::{Task, TaskStatus};
use chrono::NaiveDate;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub use icon::{TrayIcon, TrayIcons};

/// Estado visual do ícone de bandeja.
// Os nomes seguem os ícones fornecidos para o Calma ("good/attention/late
// vibes") e são mais expressivos do que nomes genéricos como "ok/alerta".
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrayState {
    /// Nenhuma tarefa atrasada ou pendente para hoje.
    #[default]
    GoodVibes,
    /// Existe pelo menos uma tarefa pendente para hoje.
    AttentionVibes,
    /// Existe pelo menos uma tarefa atrasada.
    LateVibes,
}

impl TrayState {
    /// Rótulo curto usado em menus e diagnósticos.
    pub fn label(self) -> &'static str {
        match self {
            TrayState::GoodVibes => "Tudo em dia",
            TrayState::AttentionVibes => "Tarefas para hoje",
            TrayState::LateVibes => "Tarefas atrasadas",
        }
    }

    /// Indica se o estado merece destaque na bandeja.
    pub fn needs_attention(self) -> bool {
        matches!(self, TrayState::LateVibes)
    }

    /// Calcula o estado a partir das tarefas e das conclusões por ocorrência.
    ///
    /// As regras são as mesmas usadas pela TUI e pelo agendador:
    /// tarefas recorrentes contam apenas quando ocorrem na data e a
    /// conclusão é verificada por ocorrência.
    pub fn from_tasks<'a, I>(
        tasks: I,
        completions: &HashSet<(i64, NaiveDate)>,
        today: NaiveDate,
    ) -> Self
    where
        I: IntoIterator<Item = &'a Task>,
    {
        let mut overdue = false;
        let mut today_pending = false;

        for task in tasks {
            let done = if task.is_recurring() {
                completions.contains(&(task.id, today))
            } else {
                task.status == TaskStatus::Concluida
            };

            if done {
                continue;
            }

            if !task.is_recurring() && task.due_date.is_some_and(|due| due < today) {
                overdue = true;
            }

            let occurs_today = if task.is_recurring() {
                task.occurs_on(today)
            } else {
                task.due_date == Some(today) || task.scheduled_date == Some(today)
            };

            if occurs_today {
                today_pending = true;
            }
        }

        if overdue {
            TrayState::LateVibes
        } else if today_pending {
            TrayState::AttentionVibes
        } else {
            TrayState::GoodVibes
        }
    }
}

/// Resumo exibido no ícone e no tooltip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TraySummary {
    pub state: TrayState,
    pub tasks_today: usize,
    pub overdue_tasks: usize,
    pub events_today: usize,
}

impl TraySummary {
    /// Cria o resumo a partir das contagens produzidas por uma verificação.
    pub fn from_counts(tasks_today: usize, overdue_tasks: usize, events_today: usize) -> Self {
        let state = if overdue_tasks > 0 {
            TrayState::LateVibes
        } else if tasks_today > 0 {
            TrayState::AttentionVibes
        } else {
            TrayState::GoodVibes
        };

        Self {
            state,
            tasks_today,
            overdue_tasks,
            events_today,
        }
    }

    /// Calcula o resumo diretamente das tarefas persistidas.
    ///
    /// Eventos não são contabilizados aqui; o daemon usa
    /// [`Self::from_counts`] quando já possui um relatório completo.
    pub fn from_tasks<'a, I>(
        tasks: I,
        completions: &HashSet<(i64, NaiveDate)>,
        today: NaiveDate,
    ) -> Self
    where
        I: IntoIterator<Item = &'a Task> + Clone,
    {
        let state = TrayState::from_tasks(tasks.clone(), completions, today);

        let mut tasks_today = 0;
        let mut overdue_tasks = 0;
        for task in tasks {
            let done = if task.is_recurring() {
                completions.contains(&(task.id, today))
            } else {
                task.status == TaskStatus::Concluida
            };
            if done {
                continue;
            }

            if !task.is_recurring() && task.due_date.is_some_and(|due| due < today) {
                overdue_tasks += 1;
            }

            let occurs_today = if task.is_recurring() {
                task.occurs_on(today)
            } else {
                task.due_date == Some(today) || task.scheduled_date == Some(today)
            };
            if occurs_today {
                tasks_today += 1;
            }
        }

        Self {
            state,
            tasks_today,
            overdue_tasks,
            events_today: 0,
        }
    }

    /// Título do tooltip.
    pub fn tooltip_title(&self) -> String {
        format!("Calma — {}", self.state.label())
    }

    /// Descrição do tooltip, com contagens legíveis.
    pub fn tooltip_description(&self) -> String {
        match self.state {
            TrayState::LateVibes => {
                let mut text = count(self.overdue_tasks, "tarefa atrasada", "tarefas atrasadas");
                if self.tasks_today > 0 {
                    text.push_str(" e ");
                    text.push_str(&count(
                        self.tasks_today,
                        "tarefa para hoje",
                        "tarefas para hoje",
                    ));
                }
                text
            }
            TrayState::AttentionVibes => {
                let mut text = count(self.tasks_today, "tarefa para hoje", "tarefas para hoje");
                if self.events_today > 0 {
                    text.push_str("; ");
                    text.push_str(&count(self.events_today, "evento hoje", "eventos hoje"));
                }
                text
            }
            TrayState::GoodVibes => {
                let mut text = "Nenhuma tarefa atrasada ou pendente para hoje".to_string();
                if self.events_today > 0 {
                    text.push_str("; ");
                    text.push_str(&count(self.events_today, "evento hoje", "eventos hoje"));
                }
                text
            }
        }
    }

    /// Linha curta usada como item desabilitado no menu.
    pub fn status_line(&self) -> String {
        match self.state {
            TrayState::LateVibes => {
                count(self.overdue_tasks, "tarefa atrasada", "tarefas atrasadas")
            }
            TrayState::AttentionVibes => {
                count(self.tasks_today, "tarefa para hoje", "tarefas para hoje")
            }
            TrayState::GoodVibes => "Tudo em dia".to_string(),
        }
    }
}

fn count(value: usize, singular: &str, plural: &str) -> String {
    if value == 1 {
        format!("1 {}", singular)
    } else {
        format!("{} {}", value, plural)
    }
}

impl TrayIcons {
    /// Ícone correspondente ao estado.
    pub fn for_state(&self, state: TrayState) -> &TrayIcon {
        match state {
            TrayState::GoodVibes => &self.good,
            TrayState::AttentionVibes => &self.attention,
            TrayState::LateVibes => &self.late,
        }
    }
}

/// Preferência de backend do ícone de bandeja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayPreference {
    /// Usa o backend compatível com o ambiente detectado.
    Auto,
    /// Força o StatusNotifierItem usado pelo KDE.
    Kde,
    /// Desativa o ícone sem alterar a configuração.
    Off,
}

impl TrayPreference {
    /// Interpreta o valor textual da configuração.
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "kde" | "statusnotifier" | "status-notifier" => TrayPreference::Kde,
            "off" | "none" | "noop" | "disabled" | "desativado" => TrayPreference::Off,
            _ => TrayPreference::Auto,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TrayPreference::Auto => "automático",
            TrayPreference::Kde => "KDE StatusNotifierItem",
            TrayPreference::Off => "desativado",
        }
    }
}

/// Backend concreto disponível.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayBackendKind {
    /// StatusNotifierItem do KDE/Freedesktop.
    KdeStatusNotifier,
}

impl TrayBackendKind {
    pub fn label(self) -> &'static str {
        match self {
            TrayBackendKind::KdeStatusNotifier => "KDE StatusNotifierItem",
        }
    }
}

impl fmt::Display for TrayBackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Controle de um ícone já registrado.
pub trait TrayHandle: Send {
    /// Atualiza o estado exibido.
    fn update(&mut self, summary: TraySummary);
    /// Indica se o serviço de bandeja foi encerrado.
    fn is_closed(&self) -> bool;
    /// Encerra o serviço de bandeja.
    fn shutdown(&mut self);
}

/// Opções necessárias para iniciar um backend.
#[derive(Clone)]
pub struct TrayOptions {
    pub summary: TraySummary,
    pub launcher: Arc<dyn TuiLauncher>,
    pub stop_requested: Arc<AtomicBool>,
}

impl fmt::Debug for TrayOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrayOptions")
            .field("summary", &self.summary)
            .field("launcher", &self.launcher)
            .field("stop_requested", &self.stop_requested)
            .finish()
    }
}

/// Backend de ícone de bandeja.
pub trait TrayBackend: fmt::Debug + Send + Sync {
    fn kind(&self) -> TrayBackendKind;
    /// Indica se o backend parece utilizável no ambiente atual.
    fn is_available(&self) -> bool;
    /// Registra o ícone e retorna o controle para atualizações.
    fn start(&self, options: TrayOptions) -> AppResult<Box<dyn TrayHandle>>;
}

/// Seleciona e inicia o primeiro backend disponível.
#[derive(Debug, Clone)]
pub struct TrayManager {
    backends: Vec<Arc<dyn TrayBackend>>,
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::kde()
    }
}

impl TrayManager {
    /// Gerenciador com o backend KDE.
    pub fn kde() -> Self {
        Self::with_backends(vec![Arc::new(kde::KdeTrayBackend)])
    }

    /// Cria um gerenciador com backends explícitos, na ordem de preferência.
    ///
    /// Útil para testes e para registrar futuros backends sem alterar o
    /// daemon.
    pub fn with_backends(backends: Vec<Arc<dyn TrayBackend>>) -> Self {
        Self { backends }
    }

    pub fn available_backend(&self) -> Option<TrayBackendKind> {
        self.backends
            .iter()
            .find(|backend| backend.is_available())
            .map(|backend| backend.kind())
    }

    pub fn start(&self, options: TrayOptions) -> AppResult<Box<dyn TrayHandle>> {
        let mut attempts = Vec::new();

        for backend in &self.backends {
            if !backend.is_available() {
                attempts.push(format!("{} indisponível", backend.kind().label()));
                continue;
            }

            match backend.start(options.clone()) {
                Ok(handle) => return Ok(handle),
                Err(err) => attempts.push(format!("{}: {}", backend.kind().label(), err)),
            }
        }

        Err(AppError::Tray(format!(
            "nenhum backend de bandeja compatível conseguiu iniciar. {}",
            if attempts.is_empty() {
                "Nenhum backend configurado.".to_string()
            } else {
                attempts.join(" | ")
            }
        )))
    }
}

/// Inicia a bandeja para o daemon conforme a configuração.
///
/// Retorna `Ok(None)` quando a bandeja está desativada ou quando o backend
/// automático não se aplica ao ambiente atual. Falhas de inicialização são
/// devolvidas para que o daemon possa registrar o motivo sem encerrar.
pub fn start_for_daemon(
    config: &Config,
    summary: TraySummary,
    stop_requested: Arc<AtomicBool>,
) -> AppResult<Option<Box<dyn TrayHandle>>> {
    if !config.tray.enabled {
        return Ok(None);
    }

    let preference = TrayPreference::parse(&config.tray.backend);
    if preference == TrayPreference::Off {
        return Ok(None);
    }

    let session = DesktopSession::detect();
    if preference == TrayPreference::Auto && !session.environment.is_kde() {
        return Ok(None);
    }

    let launcher = Arc::new(TerminalLauncher::for_current_exe(config, &session)?);
    let manager = TrayManager::kde();
    let options = TrayOptions {
        summary,
        launcher,
        stop_requested,
    };

    manager.start(options).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::task::{TaskRecurrence, weekday_bit};
    use chrono::NaiveTime;

    fn task(title: &str) -> Task {
        Task::new(title.to_string())
    }

    fn completions(items: &[(i64, NaiveDate)]) -> HashSet<(i64, NaiveDate)> {
        items.iter().copied().collect()
    }

    #[test]
    fn no_tasks_means_good_vibes() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
        let tasks: Vec<Task> = Vec::new();
        assert_eq!(
            TrayState::from_tasks(&tasks, &completions(&[]), today),
            TrayState::GoodVibes
        );
    }

    #[test]
    fn pending_task_today_means_attention() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
        let mut task = task("Hoje");
        task.due_date = Some(today);

        let tasks = vec![task];
        let summary = TraySummary::from_tasks(&tasks, &completions(&[]), today);

        assert_eq!(summary.state, TrayState::AttentionVibes);
        assert_eq!(summary.tasks_today, 1);
        assert_eq!(summary.overdue_tasks, 0);
    }

    #[test]
    fn overdue_task_wins_over_tasks_today() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
        let mut late = task("Atrasada");
        late.due_date = Some(today - chrono::Duration::days(1));
        let mut today_task = task("Hoje");
        today_task.due_date = Some(today);

        let tasks = vec![late, today_task];
        let summary = TraySummary::from_tasks(&tasks, &completions(&[]), today);

        assert_eq!(summary.state, TrayState::LateVibes);
        assert_eq!(summary.overdue_tasks, 1);
        assert_eq!(summary.tasks_today, 1);
    }

    #[test]
    fn completed_tasks_do_not_affect_the_icon() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
        let mut done = task("Concluída");
        done.due_date = Some(today);
        done.status = TaskStatus::Concluida;

        let tasks = vec![done];
        assert_eq!(
            TrayState::from_tasks(&tasks, &completions(&[]), today),
            TrayState::GoodVibes
        );
    }

    #[test]
    fn scheduled_task_today_counts_as_attention() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
        let mut task = task("Agendada");
        task.scheduled_date = Some(today);

        let tasks = vec![task];
        assert_eq!(
            TrayState::from_tasks(&tasks, &completions(&[]), today),
            TrayState::AttentionVibes
        );
    }

    #[test]
    fn recurring_task_respects_the_occurrence_completion() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
        let mut task = task("Diária");
        task.id = 7;
        task.recurrence = TaskRecurrence::Daily;
        task.scheduled_time = Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap());

        let tasks = vec![task];
        assert_eq!(
            TrayState::from_tasks(&tasks, &completions(&[]), today),
            TrayState::AttentionVibes
        );
        assert_eq!(
            TrayState::from_tasks(&tasks, &completions(&[(7, today)]), today),
            TrayState::GoodVibes
        );
    }

    #[test]
    fn weekly_task_only_counts_on_its_weekday() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap(); // domingo
        let mut task = task("Semanal");
        task.id = 9;
        task.recurrence = TaskRecurrence::Weekly;
        task.recurrence_days = weekday_bit(chrono::Weekday::Mon);

        let tasks = vec![task];
        assert_eq!(
            TrayState::from_tasks(&tasks, &completions(&[]), today),
            TrayState::GoodVibes
        );
    }

    #[test]
    fn from_counts_derives_state_and_tooltips() {
        let good = TraySummary::from_counts(0, 0, 0);
        assert_eq!(good.state, TrayState::GoodVibes);
        assert_eq!(good.tooltip_title(), "Calma — Tudo em dia");
        assert!(good.tooltip_description().contains("Nenhuma tarefa"));

        let attention = TraySummary::from_counts(2, 0, 1);
        assert_eq!(attention.state, TrayState::AttentionVibes);
        assert_eq!(attention.status_line(), "2 tarefas para hoje");
        assert!(attention.tooltip_description().contains("evento hoje"));

        let late = TraySummary::from_counts(1, 3, 0);
        assert_eq!(late.state, TrayState::LateVibes);
        assert_eq!(late.status_line(), "3 tarefas atrasadas");
    }

    #[test]
    fn preference_parsing_accepts_common_spellings() {
        assert_eq!(TrayPreference::parse("auto"), TrayPreference::Auto);
        assert_eq!(TrayPreference::parse("KDE"), TrayPreference::Kde);
        assert_eq!(TrayPreference::parse("statusnotifier"), TrayPreference::Kde);
        assert_eq!(TrayPreference::parse("off"), TrayPreference::Off);
        assert_eq!(TrayPreference::parse("noop"), TrayPreference::Off);
        assert_eq!(TrayPreference::parse("desconhecido"), TrayPreference::Auto);
    }

    #[test]
    fn icon_for_state_uses_the_matching_builtin_asset() {
        let icons = TrayIcons::builtin();
        for state in [
            TrayState::GoodVibes,
            TrayState::AttentionVibes,
            TrayState::LateVibes,
        ] {
            let icon = icons.for_state(state);
            assert_eq!(icon.width, 32);
            assert_eq!(icon.height, 32);
        }
    }
}
