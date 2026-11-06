use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
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
            status: TaskStatus::Pendente,
            priority: TaskPriority::Normal,
            category: None,
            sort_order: 0,
            completed_at: None,
        }
    }

    pub fn is_overdue(&self, today: NaiveDate) -> bool {
        matches!(self.status, TaskStatus::Pendente | TaskStatus::EmAndamento)
            && self.due_date.is_some_and(|d| d < today)
    }

    pub fn is_due_today(&self, today: NaiveDate) -> bool {
        matches!(self.status, TaskStatus::Pendente | TaskStatus::EmAndamento)
            && self.due_date == Some(today)
    }

    pub fn is_due_tomorrow(&self, today: NaiveDate) -> bool {
        matches!(self.status, TaskStatus::Pendente | TaskStatus::EmAndamento)
            && self
                .due_date
                .is_some_and(|d| d == today + chrono::Duration::days(1))
    }

    pub fn days_until_due(&self, today: NaiveDate) -> Option<i64> {
        self.due_date
            .map(|d| d.signed_duration_since(today).num_days())
    }

    pub fn days_overdue(&self, today: NaiveDate) -> Option<i64> {
        self.due_date
            .map(|d| today.signed_duration_since(d).num_days())
    }
}
