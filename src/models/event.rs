use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

/// Representa um evento de calendário (compromisso de ocorrência única).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub category: Option<String>,
    pub location: Option<String>,
    pub created_at: NaiveDateTime,
}

impl Event {
    pub fn new(title: String, date: NaiveDate) -> Self {
        Event {
            id: 0,
            title,
            description: None,
            date,
            start_time: None,
            end_time: None,
            category: None,
            location: None,
            created_at: chrono::Local::now().naive_local(),
        }
    }

    /// Rótulo amigável do horário do evento.
    pub fn time_label(&self) -> String {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => {
                format!("{}-{}", start.format("%H:%M"), end.format("%H:%M"))
            }
            (Some(start), None) => start.format("%H:%M").to_string(),
            _ => "—".to_string(),
        }
    }

    /// Indica se o evento já aconteceu.
    pub fn is_past(&self, today: NaiveDate) -> bool {
        self.date < today
    }
}
