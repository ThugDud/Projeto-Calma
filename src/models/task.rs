use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

/// Estado de uma tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pendente,
    EmAndamento,
    Concluida,
}

/// Prioridade de uma tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Baixa,
    Normal,
    Alta,
    Urgente,
}

/// Regra de repetição de uma tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskRecurrence {
    /// Tarefa única, sem repetição.
    #[default]
    None,
    /// Repete todos os dias.
    Daily,
    /// Repete nos dias da semana indicados em `recurrence_days`.
    Weekly,
}

impl TaskRecurrence {
    pub fn is_recurring(self) -> bool {
        !matches!(self, TaskRecurrence::None)
    }

    pub fn label(self) -> &'static str {
        match self {
            TaskRecurrence::None => "Nenhuma",
            TaskRecurrence::Daily => "Diária",
            TaskRecurrence::Weekly => "Semanal",
        }
    }

    pub fn as_i32(self) -> i32 {
        match self {
            TaskRecurrence::None => 0,
            TaskRecurrence::Daily => 1,
            TaskRecurrence::Weekly => 2,
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => TaskRecurrence::Daily,
            2 => TaskRecurrence::Weekly,
            _ => TaskRecurrence::None,
        }
    }
}

/// Todos os dias úteis e fins de semana em um mapa de bits, com segunda como bit 0.
pub const ALL_WEEKDAYS: u8 = 0b111_1111;
/// Máscara padrão ao ativar uma repetição semanal: segunda a sexta.
pub const WEEKDAYS_MON_TO_FRI: u8 = 0b001_1111;

/// Ordem de exibição dos dias da semana (segunda a domingo).
pub const WEEKDAY_ORDER: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

/// Retorna o bit correspondente ao dia da semana.
pub fn weekday_bit(weekday: Weekday) -> u8 {
    1 << weekday.num_days_from_monday()
}

/// Converte um índice de 0 a 6 (segunda a domingo) no dia da semana.
pub fn weekday_from_index(index: usize) -> Option<Weekday> {
    WEEKDAY_ORDER.get(index).copied()
}

/// Rótulo curto do dia da semana em português.
pub fn weekday_short(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Seg",
        Weekday::Tue => "Ter",
        Weekday::Wed => "Qua",
        Weekday::Thu => "Qui",
        Weekday::Fri => "Sex",
        Weekday::Sat => "Sáb",
        Weekday::Sun => "Dom",
    }
}

/// Formata uma máscara de dias como "Seg, Qua, Sex".
pub fn recurrence_days_label(days: u8) -> String {
    let selected: Vec<&str> = WEEKDAY_ORDER
        .iter()
        .filter(|weekday| days & weekday_bit(**weekday) != 0)
        .map(|weekday| weekday_short(*weekday))
        .collect();

    if selected.is_empty() {
        "Nenhum dia".to_string()
    } else {
        selected.join(", ")
    }
}

/// Representa uma tarefa no sistema Calma.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
    pub project_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub due_date: Option<NaiveDate>,
    pub scheduled_date: Option<NaiveDate>,
    pub scheduled_time: Option<NaiveTime>,
    /// Repetição da tarefa. Ausente em backups antigos, assume `None`.
    #[serde(default)]
    pub recurrence: TaskRecurrence,
    /// Dias da semana da repetição semanal (segunda = bit 0).
    #[serde(default)]
    pub recurrence_days: u8,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub category: Option<String>,
    pub sort_order: i32,
    pub completed_at: Option<NaiveDateTime>,
}

impl Task {
    pub fn new(title: String) -> Self {
        Task {
            id: 0,
            title,
            description: None,
            parent_id: None,
            project_id: None,
            created_at: chrono::Local::now().naive_local(),
            due_date: None,
            scheduled_date: None,
            scheduled_time: None,
            recurrence: TaskRecurrence::None,
            recurrence_days: 0,
            status: TaskStatus::Pendente,
            priority: TaskPriority::Normal,
            category: None,
            sort_order: 0,
            completed_at: None,
        }
    }

    /// Indica se a tarefa possui alguma regra de repetição.
    pub fn is_recurring(&self) -> bool {
        self.recurrence.is_recurring()
    }

    /// Indica se a tarefa ocorre em uma data específica.
    ///
    /// Tarefas únicas não são consideradas recorrentes; para elas, use as
    /// datas de vencimento/agendamento. Tarefas recorrentes sempre ocorrem
    /// na data da regra, independentemente de `due_date`.
    pub fn occurs_on(&self, date: NaiveDate) -> bool {
        match self.recurrence {
            TaskRecurrence::None => false,
            TaskRecurrence::Daily => true,
            TaskRecurrence::Weekly => self.recurrence_days & weekday_bit(date.weekday()) != 0,
        }
    }

    /// Rótulo amigável da repetição, com horário quando definido.
    pub fn recurrence_label(&self) -> Option<String> {
        let mut label = match self.recurrence {
            TaskRecurrence::None => return None,
            TaskRecurrence::Daily => "Diária".to_string(),
            TaskRecurrence::Weekly => {
                format!("Semanal: {}", recurrence_days_label(self.recurrence_days))
            }
        };

        if let Some(time) = self.scheduled_time {
            label.push_str(&format!(" às {}", time.format("%H:%M")));
        }
        Some(label)
    }

    pub fn is_overdue(&self, today: NaiveDate) -> bool {
        !self.is_recurring()
            && matches!(self.status, TaskStatus::Pendente | TaskStatus::EmAndamento)
            && self.due_date.is_some_and(|d| d < today)
    }

    pub fn is_due_today(&self, today: NaiveDate) -> bool {
        !self.is_recurring()
            && matches!(self.status, TaskStatus::Pendente | TaskStatus::EmAndamento)
            && self.due_date == Some(today)
    }

    pub fn is_due_tomorrow(&self, today: NaiveDate) -> bool {
        !self.is_recurring()
            && matches!(self.status, TaskStatus::Pendente | TaskStatus::EmAndamento)
            && self
                .due_date
                .is_some_and(|d| d == today + chrono::Duration::days(1))
    }

    pub fn days_until_due(&self, today: NaiveDate) -> Option<i64> {
        if self.is_recurring() {
            return None;
        }
        self.due_date
            .map(|d| d.signed_duration_since(today).num_days())
    }

    pub fn days_overdue(&self, today: NaiveDate) -> Option<i64> {
        if self.is_recurring() {
            return None;
        }
        self.due_date
            .map(|d| today.signed_duration_since(d).num_days())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn daily_task_occurs_every_day() {
        let mut task = Task::new("Academia".into());
        task.recurrence = TaskRecurrence::Daily;

        assert!(task.is_recurring());
        assert!(task.occurs_on(date(2026, 11, 9)));
        assert!(task.occurs_on(date(2026, 11, 10)));
        assert_eq!(task.recurrence_label().as_deref(), Some("Diária"));
    }

    #[test]
    fn weekly_task_occurs_only_on_selected_weekdays() {
        let mut task = Task::new("Revisão".into());
        task.recurrence = TaskRecurrence::Weekly;
        task.recurrence_days = WEEKDAYS_MON_TO_FRI;

        // 2026-11-09 é segunda-feira.
        assert!(task.occurs_on(date(2026, 11, 9)));
        assert!(task.occurs_on(date(2026, 11, 13)));
        assert!(!task.occurs_on(date(2026, 11, 14)));
        assert_eq!(
            task.recurrence_label().as_deref(),
            Some("Semanal: Seg, Ter, Qua, Qui, Sex")
        );
    }

    #[test]
    fn recurrence_label_includes_scheduled_time() {
        let mut task = Task::new("Corrida".into());
        task.recurrence = TaskRecurrence::Daily;
        task.scheduled_time = NaiveTime::from_hms_opt(6, 30, 0);
        assert_eq!(task.recurrence_label().as_deref(), Some("Diária às 06:30"));
    }

    #[test]
    fn recurring_task_is_never_overdue_or_due_today() {
        let mut task = Task::new("Rotina".into());
        task.recurrence = TaskRecurrence::Daily;
        task.due_date = Some(date(2026, 11, 1));

        assert!(!task.is_overdue(date(2026, 11, 10)));
        assert!(!task.is_due_today(date(2026, 11, 10)));
        assert_eq!(task.days_until_due(date(2026, 11, 10)), None);
        assert_eq!(task.days_overdue(date(2026, 11, 10)), None);
    }

    #[test]
    fn weekday_helpers_map_monday_to_first_bit() {
        assert_eq!(weekday_bit(Weekday::Mon), 0b000_0001);
        assert_eq!(weekday_bit(Weekday::Sun), 0b100_0000);
        assert_eq!(weekday_from_index(0), Some(Weekday::Mon));
        assert_eq!(weekday_from_index(6), Some(Weekday::Sun));
        assert_eq!(weekday_from_index(7), None);
        assert_eq!(
            recurrence_days_label(WEEKDAYS_MON_TO_FRI),
            "Seg, Ter, Qua, Qui, Sex"
        );
        assert_eq!(recurrence_days_label(0), "Nenhum dia");
    }

    #[test]
    fn recurrence_values_roundtrip() {
        for rule in [
            TaskRecurrence::None,
            TaskRecurrence::Daily,
            TaskRecurrence::Weekly,
        ] {
            assert_eq!(TaskRecurrence::from_i32(rule.as_i32()), rule);
        }
        assert_eq!(TaskRecurrence::from_i32(99), TaskRecurrence::None);
    }
}
