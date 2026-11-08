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
    /// Tarefas não concluídas com vencimento ou agendamento no dia atual.
    pub tasks_today: Vec<Task>,
    /// Tarefas não concluídas cujo vencimento já passou.
    pub overdue_tasks: Vec<Task>,
    /// Eventos que acontecem hoje e ainda não terminaram.
    pub events_today: Vec<Event>,
    /// Eventos com horário de início dentro da antecedência configurada.
    pub events_starting_soon: Vec<Event>,
}

impl CheckReport {
    pub fn is_empty(&self) -> bool {
        self.tasks_today.is_empty()
            && self.overdue_tasks.is_empty()
            && self.events_today.is_empty()
            && self.events_starting_soon.is_empty()
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
    /// Tarefas concluídas e eventos já encerrados são ignorados. Tarefas e
    /// eventos são considerados "de hoje" quando a data atual coincide com o
    /// vencimento, o agendamento ou a data do evento. Eventos com horário de
    /// início dentro de `notifications.advance_minutes` também são expostos
    /// separadamente para o aviso de antecedência.
    pub fn check(
        &self,
        conn: &Connection,
        config: &Config,
        now: NaiveDateTime,
    ) -> AppResult<CheckReport> {
        let today = now.date();
        let lead_minutes = config.notifications.advance_minutes.clamp(
            crate::config::MIN_NOTIFICATION_ADVANCE_MINUTES,
            MAX_NOTIFICATION_ADVANCE_MINUTES,
        ) as i64;
        let soon_horizon = now + Duration::minutes(lead_minutes);

        let tasks = db::list_tasks(conn)?;
        let mut tasks_today: Vec<Task> = tasks
            .iter()
            .filter(|task| {
                task.status != TaskStatus::Concluida
                    && (task.due_date == Some(today) || task.scheduled_date == Some(today))
            })
            .cloned()
            .collect();
        tasks_today.sort_by_key(|task| {
            (
                task.due_date,
                task.scheduled_time,
                task.title.to_lowercase(),
            )
        });

        let mut overdue_tasks: Vec<Task> = tasks
            .iter()
            .filter(|task| {
                task.status != TaskStatus::Concluida && task.due_date.is_some_and(|due| due < today)
            })
            .cloned()
            .collect();
        overdue_tasks.sort_by_key(|task| (task.due_date, task.title.to_lowercase()));

        let events = db::list_events(conn)?;
        let mut events_today: Vec<Event> = events
            .iter()
            .filter(|event| event.date == today && !has_ended(event, now))
            .cloned()
            .collect();
        events_today
            .sort_by_key(|event| (event.date, event.start_time, event.title.to_lowercase()));

        let mut events_starting_soon: Vec<Event> = events
            .iter()
            .filter(|event| match event.start_time {
                Some(start_time) => {
                    let starts_at = event.date.and_time(start_time);
                    starts_at > now && starts_at <= soon_horizon
                }
                None => false,
            })
            .cloned()
            .collect();
        events_starting_soon
            .sort_by_key(|event| (event.date, event.start_time, event.title.to_lowercase()));

        Ok(CheckReport {
            checked_at: Some(now),
            tasks_today,
            overdue_tasks,
            events_today,
            events_starting_soon,
        })
    }
}

/// Um evento é considerado encerrado quando possui horário de fim e esse
/// horário já passou. Eventos sem horário de fim permanecem válidos para o
/// aviso do dia, mesmo que o horário de início já tenha passado.
fn has_ended(event: &Event, now: NaiveDateTime) -> bool {
    event
        .end_time
        .is_some_and(|end| event.date.and_time(end) <= now)
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

    fn fixed_now() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 5)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap()
    }

    fn event_at(title: &str, date: NaiveDate, hour: u32, minute: u32) -> Event {
        let mut event = Event::new(title.into(), date);
        event.start_time = Some(NaiveTime::from_hms_opt(hour, minute, 0).unwrap());
        event
    }

    #[test]
    fn check_reports_tasks_due_or_scheduled_today_and_overdue() {
        let conn = db::init_memory().unwrap();
        let now = fixed_now();
        let today = now.date();

        let mut overdue = Task::new("Atrasada".into());
        overdue.due_date = Some(today - Duration::days(1));
        db::insert_task(&conn, &overdue).unwrap();

        let mut due_today = Task::new("Vence hoje".into());
        due_today.due_date = Some(today);
        db::insert_task(&conn, &due_today).unwrap();

        let mut scheduled_today = Task::new("Agendada hoje".into());
        scheduled_today.scheduled_date = Some(today);
        scheduled_today.due_date = Some(today + Duration::days(3));
        db::insert_task(&conn, &scheduled_today).unwrap();

        let mut completed = Task::new("Concluída".into());
        completed.due_date = Some(today);
        completed.status = TaskStatus::Concluida;
        db::insert_task(&conn, &completed).unwrap();

        let mut tomorrow = Task::new("Amanhã".into());
        tomorrow.due_date = Some(today + Duration::days(1));
        db::insert_task(&conn, &tomorrow).unwrap();

        let config = test_config(60);
        let scheduler = Scheduler::from_config(&config);
        let report = scheduler
            .check(&conn, &config, now)
            .expect("verificação deve funcionar");

        let today_titles: Vec<&str> = report
            .tasks_today
            .iter()
            .map(|task| task.title.as_str())
            .collect();
        assert_eq!(today_titles, vec!["Vence hoje", "Agendada hoje"]);
        assert_eq!(report.overdue_tasks.len(), 1);
        assert_eq!(report.overdue_tasks[0].title, "Atrasada");
    }

    #[test]
    fn check_reports_events_today_and_starting_soon_separately() {
        let conn = db::init_memory().unwrap();
        let now = fixed_now();
        let today = now.date();

        let soon = event_at("Reunião", today, 10, 20);
        db::insert_event(&conn, &soon).unwrap();

        let all_day = Event::new("Dia inteiro".into(), today);
        db::insert_event(&conn, &all_day).unwrap();

        let later = event_at("Tarde", today, 15, 0);
        db::insert_event(&conn, &later).unwrap();

        let mut ended = event_at("Já terminou", today, 9, 0);
        ended.end_time = Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap());
        db::insert_event(&conn, &ended).unwrap();

        let past = Event::new("Ontem".into(), today - Duration::days(1));
        db::insert_event(&conn, &past).unwrap();

        let tomorrow = event_at("Amanhã", today + Duration::days(1), 11, 0);
        db::insert_event(&conn, &tomorrow).unwrap();

        let config = test_config(30);
        let scheduler = Scheduler::from_config(&config);
        let report = scheduler
            .check(&conn, &config, now)
            .expect("verificação deve funcionar");

        let today_titles: Vec<&str> = report
            .events_today
            .iter()
            .map(|event| event.title.as_str())
            .collect();
        assert_eq!(today_titles, vec!["Dia inteiro", "Reunião", "Tarde"]);

        let soon_titles: Vec<&str> = report
            .events_starting_soon
            .iter()
            .map(|event| event.title.as_str())
            .collect();
        assert_eq!(soon_titles, vec!["Reunião"]);
    }

    #[test]
    fn check_starting_soon_window_uses_configured_lead_time() {
        let conn = db::init_memory().unwrap();
        let now = fixed_now();
        let today = now.date();

        let boundary = event_at("No limite", today, 11, 0);
        db::insert_event(&conn, &boundary).unwrap();

        let after = event_at("Depois", today, 11, 1);
        db::insert_event(&conn, &after).unwrap();

        let now_event = event_at("Agora", today, 10, 0);
        db::insert_event(&conn, &now_event).unwrap();

        let config = test_config(60);
        let scheduler = Scheduler::from_config(&config);
        let report = scheduler
            .check(&conn, &config, now)
            .expect("verificação deve funcionar");

        let soon_titles: Vec<&str> = report
            .events_starting_soon
            .iter()
            .map(|event| event.title.as_str())
            .collect();
        assert_eq!(soon_titles, vec!["No limite"]);
    }
}
