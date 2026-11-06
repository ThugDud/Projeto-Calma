use crate::config::{
    Config, MAX_NOTIFICATION_ADVANCE_MINUTES, MAX_POLL_INTERVAL_SECS, MIN_POLL_INTERVAL_SECS,
};
use crate::db;
use crate::errors::AppResult;
use crate::models::event::Event;
use crate::models::task::{Task, TaskStatus};
use chrono::{Duration, NaiveDateTime};
use rusqlite::Connection;
use std::time::{Duration as StdDuration, Instant};

/// Resultado de uma verificação periódica do daemon.
///
/// A estrutura é intencionalmente simples: ela apenas reúne os itens que
/// merecem atenção. A decisão de quais notificações gerar fica no módulo
/// `notifications`, e a entrega fica atrás da abstração `Notifier`.
#[derive(Debug, Clone, Default)]
pub struct CheckReport {
    pub checked_at: Option<NaiveDateTime>,
    pub upcoming_tasks: Vec<Task>,
    pub overdue_tasks: Vec<Task>,
    pub upcoming_events: Vec<Event>,
}

impl CheckReport {
    pub fn is_empty(&self) -> bool {
        self.upcoming_tasks.is_empty()
            && self.overdue_tasks.is_empty()
            && self.upcoming_events.is_empty()
    }
}

/// Agendador básico baseado em intervalo.
///
/// Não há cron, threads ou regras complexas. O objetivo é oferecer um ponto
/// único para decidir quando executar uma verificação e o que verificar.
#[derive(Debug, Clone)]
pub struct Scheduler {
    interval: StdDuration,
    last_check: Option<Instant>,
}

impl Scheduler {
    /// Cria um agendador a partir da configuração centralizada.
    pub fn from_config(config: &Config) -> Self {
        let secs = config
            .general
            .poll_interval_secs
            .clamp(MIN_POLL_INTERVAL_SECS, MAX_POLL_INTERVAL_SECS);
        Self {
            interval: StdDuration::from_secs(secs),
            last_check: None,
        }
    }

    /// Intervalo configurado entre verificações.
    pub fn interval(&self) -> StdDuration {
        self.interval
    }

    /// Indica se já é hora de executar outra verificação.
    pub fn should_check(&self, now: Instant) -> bool {
        match self.last_check {
            Some(last) => now.duration_since(last) >= self.interval,
            None => true,
        }
    }

    /// Registra que uma verificação foi concluída.
    pub fn mark_checked(&mut self, now: Instant) {
        self.last_check = Some(now);
    }

    /// Executa uma verificação única de prazos e eventos.
    ///
    /// Tarefas e eventos concluídos/passados são ignorados. O horizonte de
    /// antecedência vem de `notifications.advance_minutes`.
    pub fn check(
        &self,
        conn: &Connection,
        config: &Config,
        now: NaiveDateTime,
    ) -> AppResult<CheckReport> {
        let today = now.date();
        let advance = config.notifications.advance_minutes.clamp(
            crate::config::MIN_NOTIFICATION_ADVANCE_MINUTES,
            MAX_NOTIFICATION_ADVANCE_MINUTES,
        ) as i64;
        let advance_days = advance / (24 * 60);
        let task_horizon = today + Duration::days(advance_days);

        let tasks = db::list_tasks(conn)?;
        let mut upcoming_tasks: Vec<Task> = tasks
            .iter()
            .filter(|task| {
                task.status != TaskStatus::Concluida
                    && task
                        .due_date
                        .is_some_and(|due| due >= today && due <= task_horizon)
            })
            .cloned()
            .collect();
        upcoming_tasks.sort_by_key(|task| (task.due_date, task.title.to_lowercase()));

        let mut overdue_tasks: Vec<Task> = tasks
            .iter()
            .filter(|task| {
                task.status != TaskStatus::Concluida && task.due_date.is_some_and(|due| due < today)
            })
            .cloned()
            .collect();
        overdue_tasks.sort_by_key(|task| (task.due_date, task.title.to_lowercase()));

        let notification_horizon = now + Duration::minutes(advance);
        let events = db::list_events(conn)?;
        let mut upcoming_events: Vec<Event> = events
            .into_iter()
            .filter(|event| match event.start_time {
                Some(start_time) => {
                    let starts_at = event.date.and_time(start_time);
                    starts_at >= now && starts_at <= notification_horizon
                }
                // Eventos sem horário são tratados como eventos de dia inteiro.
                None => event.date >= today && event.date <= task_horizon,
            })
            .collect();
        upcoming_events.sort_by_key(|event| (event.date, event.start_time, event.title.clone()));

        Ok(CheckReport {
            checked_at: Some(now),
            upcoming_tasks,
            overdue_tasks,
            upcoming_events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::event::Event;
    use crate::models::task::Task;
    use chrono::{NaiveDate, NaiveTime};

    fn test_config(advance_minutes: u32) -> Config {
        let mut config = Config::default();
        config.notifications.advance_minutes = advance_minutes;
        config.general.poll_interval_secs = 30;
        config
    }

    #[test]
    fn scheduler_uses_configured_interval() {
        let config = test_config(30);
        let scheduler = Scheduler::from_config(&config);

        assert_eq!(scheduler.interval(), StdDuration::from_secs(30));
        assert!(scheduler.should_check(Instant::now()));
    }

    #[test]
    fn scheduler_clamps_invalid_interval() {
        let mut config = Config::default();
        config.general.poll_interval_secs = 0;
        let scheduler = Scheduler::from_config(&config);
        assert_eq!(
            scheduler.interval(),
            StdDuration::from_secs(MIN_POLL_INTERVAL_SECS)
        );
    }

    #[test]
    fn check_finds_upcoming_and_overdue_items() {
        let conn = db::init_memory().unwrap();
        let now = NaiveDate::from_ymd_opt(2026, 11, 5)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let today = now.date();

        let mut overdue = Task::new("Atrasada".into());
        overdue.due_date = Some(today - Duration::days(1));
        db::insert_task(&conn, &overdue).unwrap();

        let mut upcoming = Task::new("Hoje".into());
        upcoming.due_date = Some(today);
        db::insert_task(&conn, &upcoming).unwrap();

        let mut completed = Task::new("Concluída".into());
        completed.due_date = Some(today);
        completed.status = TaskStatus::Concluida;
        db::insert_task(&conn, &completed).unwrap();

        let event_start = now + Duration::minutes(20);
        let mut event = Event::new("Reunião".into(), event_start.date());
        event.start_time = Some(NaiveTime::from_hms_opt(10, 20, 0).unwrap());
        db::insert_event(&conn, &event).unwrap();

        let past_event = Event::new("Evento passado".into(), today - Duration::days(2));
        db::insert_event(&conn, &past_event).unwrap();

        let scheduler = Scheduler::from_config(&test_config(30));
        let report = scheduler
            .check(&conn, &test_config(30), now)
            .expect("verificação deve funcionar");

        assert_eq!(report.upcoming_tasks.len(), 1);
        assert_eq!(report.upcoming_tasks[0].title, "Hoje");
        assert_eq!(report.overdue_tasks.len(), 1);
        assert_eq!(report.overdue_tasks[0].title, "Atrasada");
        assert_eq!(report.upcoming_events.len(), 1);
        assert_eq!(report.upcoming_events[0].title, "Reunião");
    }
}
