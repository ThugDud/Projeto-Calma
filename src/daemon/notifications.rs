use crate::config::NotificationSettings;
use crate::daemon::scheduler::CheckReport;
pub use crate::desktop::notifications::noop::NoopNotifier;
pub use crate::desktop::notifications::notifier::{
    Notification, NotificationError, NotificationKind, Notifier,
};
use chrono::NaiveDate;
use std::collections::HashMap;

/// Serviço responsável por transformar verificações em notificações e
/// encaminhá-las para um `Notifier`.
///
/// A deduplicação é mantida apenas em memória. Cada chave entregue com
/// sucesso fica registrada até sua data de expiração, evitando que o mesmo
/// aviso seja repetido a cada verificação do daemon.
#[derive(Debug, Clone)]
pub struct NotificationService {
    enabled: bool,
    delivered_keys: HashMap<String, Option<NaiveDate>>,
    tracking_day: Option<NaiveDate>,
}

impl NotificationService {
    pub fn from_config(config: &NotificationSettings) -> Self {
        Self {
            enabled: config.enabled,
            delivered_keys: HashMap::new(),
            tracking_day: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Descarta chaves de deduplicação que já não podem ser reutilizadas.
    ///
    /// Chaves sem expiração são mantidas. Chaves de eventos que cruzam a
    /// meia-noite permanecem válidas até a data do evento, evitando que o
    /// aviso de antecedência seja repetido no dia seguinte.
    pub fn begin_day(&mut self, day: NaiveDate) {
        if self.tracking_day != Some(day) {
            self.delivered_keys
                .retain(|_, expires_on| expires_on.is_none_or(|date| date >= day));
            self.tracking_day = Some(day);
        }
    }

    /// Gera notificações respeitando as preferências do usuário.
    pub fn build(&self, report: &CheckReport, config: &NotificationSettings) -> Vec<Notification> {
        if !self.enabled || !config.enabled {
            return Vec::new();
        }

        let now = report.checked_at;
        let day = now
            .map(|now| now.date())
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        let mut notifications = Vec::new();

        if config.overdue_tasks {
            for task in &report.overdue_tasks {
                let body = match task.due_date {
                    Some(due) => format!(
                        "A tarefa \"{}\" venceu em {}.",
                        task.title,
                        due.format("%d/%m/%Y")
                    ),
                    None => format!("A tarefa \"{}\" está atrasada.", task.title),
                };
                notifications.push(
                    Notification::new(NotificationKind::OverdueTask, "Tarefa atrasada", body)
                        .with_key_until(format!("task-overdue:{}:{}", task.id, day), day),
                );
            }
        }

        if config.upcoming_tasks {
            for task in &report.tasks_today {
                let mut reasons = Vec::new();
                if task.due_date == Some(day) {
                    reasons.push("vence hoje");
                }
                if task.scheduled_date == Some(day) {
                    reasons.push("está agendada para hoje");
                }

                let reason = if reasons.is_empty() {
                    "está prevista para hoje".to_string()
                } else {
                    reasons.join(" e ")
                };

                notifications.push(
                    Notification::new(
                        NotificationKind::UpcomingTask,
                        "Tarefa para hoje",
                        format!("A tarefa \"{}\" {}.", task.title, reason),
                    )
                    .with_key_until(format!("task-today:{}:{}", task.id, day), day),
                );
            }
        }

        if config.upcoming_events {
            for event in &report.events_today {
                let when = event
                    .start_time
                    .map(|time| format!("às {}", time.format("%H:%M")))
                    .unwrap_or_else(|| "durante todo o dia".to_string());

                notifications.push(
                    Notification::new(
                        NotificationKind::UpcomingEvent,
                        "Evento hoje",
                        format!(
                            "O evento \"{}\" acontece hoje ({}) {}.",
                            event.title,
                            event.date.format("%d/%m/%Y"),
                            when
                        ),
                    )
                    .with_key_until(
                        format!("event-today:{}:{}", event.id, event.date),
                        event.date,
                    ),
                );
            }

            for event in &report.events_starting_soon {
                let Some(start_time) = event.start_time else {
                    continue;
                };
                let starts_at = event.date.and_time(start_time);
                let body = match now {
                    Some(now) => {
                        let minutes = (starts_at - now).num_minutes().max(0);
                        let remaining = if minutes <= 1 {
                            "em menos de 1 minuto".to_string()
                        } else {
                            format!("em {} minutos", minutes)
                        };
                        format!("O evento \"{}\" começa {}.", event.title, remaining)
                    }
                    None => format!(
                        "O evento \"{}\" começa às {}.",
                        event.title,
                        start_time.format("%H:%M")
                    ),
                };

                notifications.push(
                    Notification::new(NotificationKind::UpcomingEvent, "Evento em breve", body)
                        .with_key_until(
                            format!("event-soon:{}:{}:{}", event.id, event.date, start_time),
                            event.date,
                        ),
                );
            }
        }

        notifications
    }

    /// Encaminha notificações para o entregador informado.
    ///
    /// Notificações com `dedup_key` já entregues são ignoradas. A chave só é
    /// registrada depois de uma entrega bem-sucedida, permitindo nova
    /// tentativa na próxima verificação em caso de falha. Retorna quantas
    /// notificações foram realmente entregues nesta chamada.
    pub fn dispatch(
        &mut self,
        notifications: &[Notification],
        notifier: &mut dyn Notifier,
    ) -> Result<usize, NotificationError> {
        if !self.enabled {
            return Ok(0);
        }

        let mut delivered = 0;
        for notification in notifications {
            if let Some(key) = notification.dedup_key.as_deref() {
                if self.delivered_keys.contains_key(key) {
                    continue;
                }
            }

            notifier.notify(notification)?;

            if let Some(key) = notification.dedup_key.as_deref() {
                self.delivered_keys
                    .insert(key.to_string(), notification.dedup_expires_on);
            }
            delivered += 1;
        }
        Ok(delivered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NotificationSettings;
    use crate::daemon::scheduler::CheckReport;
    use crate::models::event::Event;
    use crate::models::task::Task;
    use chrono::{NaiveDate, NaiveTime};

    #[derive(Default)]
    struct RecordingNotifier {
        delivered: Vec<Notification>,
    }

    impl Notifier for RecordingNotifier {
        fn notify(&mut self, notification: &Notification) -> Result<(), NotificationError> {
            self.delivered.push(notification.clone());
            Ok(())
        }
    }

    struct FailingNotifier;

    impl Notifier for FailingNotifier {
        fn notify(&mut self, _notification: &Notification) -> Result<(), NotificationError> {
            Err(NotificationError::Delivery("falha simulada".into()))
        }
    }

    #[derive(Default)]
    struct FailingOnceNotifier {
        failed: bool,
        delivered: usize,
    }

    impl Notifier for FailingOnceNotifier {
        fn notify(&mut self, _notification: &Notification) -> Result<(), NotificationError> {
            if !self.failed {
                self.failed = true;
                return Err(NotificationError::Delivery("falha simulada".into()));
            }
            self.delivered += 1;
            Ok(())
        }
    }

    fn fixed_now() -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 5)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap()
    }

    fn report_with_items() -> CheckReport {
        let mut overdue = Task::new("Conta atrasada".into());
        overdue.due_date = NaiveDate::from_ymd_opt(2026, 11, 4);

        let mut upcoming = Task::new("Prova".into());
        upcoming.due_date = NaiveDate::from_ymd_opt(2026, 11, 5);

        CheckReport {
            checked_at: Some(fixed_now()),
            tasks_today: vec![upcoming],
            overdue_tasks: vec![overdue],
            events_today: Vec::new(),
            events_starting_soon: Vec::new(),
        }
    }

    fn report_with_event() -> CheckReport {
        let now = fixed_now();
        let mut event = Event::new("Reunião".into(), now.date());
        event.start_time = Some(NaiveTime::from_hms_opt(10, 30, 0).unwrap());

        CheckReport {
            checked_at: Some(now),
            tasks_today: Vec::new(),
            overdue_tasks: Vec::new(),
            events_today: vec![event.clone()],
            events_starting_soon: vec![event],
        }
    }

    #[test]
    fn notifications_are_disabled_by_default() {
        let config = NotificationSettings::default();
        let service = NotificationService::from_config(&config);
        let report = report_with_items();

        assert!(!service.is_enabled());
        assert!(service.build(&report, &config).is_empty());
    }

    #[test]
    fn notification_categories_are_respected() {
        let mut config = NotificationSettings {
            enabled: true,
            upcoming_tasks: false,
            overdue_tasks: true,
            upcoming_events: true,
            advance_minutes: 30,
            ..NotificationSettings::default()
        };
        let service = NotificationService::from_config(&config);
        let report = report_with_items();

        let notifications = service.build(&report, &config);
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].kind, NotificationKind::OverdueTask);

        config.overdue_tasks = false;
        assert!(service.build(&report, &config).is_empty());
    }

    #[test]
    fn dispatch_uses_notifier_abstraction() {
        let config = NotificationSettings {
            enabled: true,
            upcoming_tasks: true,
            overdue_tasks: true,
            upcoming_events: true,
            ..NotificationSettings::default()
        };
        let mut service = NotificationService::from_config(&config);
        let report = report_with_items();
        let notifications = service.build(&report, &config);
        let mut notifier = RecordingNotifier::default();

        let delivered = service
            .dispatch(&notifications, &mut notifier)
            .expect("dispatch deve funcionar");

        assert_eq!(delivered, notifications.len());
        assert_eq!(notifier.delivered.len(), notifications.len());
    }

    #[test]
    fn build_creates_distinct_keys_for_day_and_pre_event_notifications() {
        let config = NotificationSettings {
            enabled: true,
            upcoming_events: true,
            advance_minutes: 60,
            ..NotificationSettings::default()
        };
        let service = NotificationService::from_config(&config);
        let report = report_with_event();

        let notifications = service.build(&report, &config);

        assert_eq!(notifications.len(), 2);
        assert_eq!(notifications[0].title, "Evento hoje");
        assert_eq!(notifications[1].title, "Evento em breve");
        assert!(
            notifications
                .iter()
                .all(|notification| notification.dedup_key.is_some())
        );
        assert_ne!(notifications[0].dedup_key, notifications[1].dedup_key);
    }

    #[test]
    fn dispatch_skips_notifications_already_delivered() {
        let config = NotificationSettings {
            enabled: true,
            upcoming_tasks: true,
            overdue_tasks: true,
            ..NotificationSettings::default()
        };
        let mut service = NotificationService::from_config(&config);
        let report = report_with_items();
        let notifications = service.build(&report, &config);
        let mut notifier = RecordingNotifier::default();

        let first = service
            .dispatch(&notifications, &mut notifier)
            .expect("primeiro dispatch deve funcionar");
        let second = service
            .dispatch(&notifications, &mut notifier)
            .expect("segundo dispatch deve funcionar");

        assert_eq!(first, notifications.len());
        assert_eq!(second, 0);
        assert_eq!(notifier.delivered.len(), notifications.len());
    }

    #[test]
    fn failed_delivery_is_retried_on_the_next_dispatch() {
        let config = NotificationSettings {
            enabled: true,
            overdue_tasks: true,
            ..NotificationSettings::default()
        };
        let mut service = NotificationService::from_config(&config);
        let report = report_with_items();
        let notifications = service.build(&report, &config);
        let mut notifier = FailingOnceNotifier::default();

        assert!(service.dispatch(&notifications, &mut notifier).is_err());
        assert_eq!(notifier.delivered, 0);

        let delivered = service
            .dispatch(&notifications, &mut notifier)
            .expect("nova tentativa deve funcionar");

        assert_eq!(delivered, notifications.len());
        assert_eq!(notifier.delivered, notifications.len());
    }

    #[test]
    fn pre_event_notification_survives_midnight_until_event_date() {
        let now = NaiveDate::from_ymd_opt(2026, 11, 5)
            .unwrap()
            .and_hms_opt(23, 45, 0)
            .unwrap();
        let event_date = NaiveDate::from_ymd_opt(2026, 11, 6).unwrap();
        let mut event = Event::new("Virada".into(), event_date);
        event.start_time = Some(NaiveTime::from_hms_opt(0, 30, 0).unwrap());

        let report = CheckReport {
            checked_at: Some(now),
            tasks_today: Vec::new(),
            overdue_tasks: Vec::new(),
            events_today: Vec::new(),
            events_starting_soon: vec![event],
        };
        let config = NotificationSettings {
            enabled: true,
            upcoming_events: true,
            advance_minutes: 60,
            ..NotificationSettings::default()
        };
        let mut service = NotificationService::from_config(&config);
        let notifications = service.build(&report, &config);
        let mut notifier = RecordingNotifier::default();

        service.begin_day(now.date());
        assert_eq!(service.dispatch(&notifications, &mut notifier).unwrap(), 1);

        service.begin_day(event_date);
        assert_eq!(service.dispatch(&notifications, &mut notifier).unwrap(), 0);
        assert_eq!(notifier.delivered.len(), 1);
    }

    #[test]
    fn begin_day_clears_deduplication_state() {
        let config = NotificationSettings {
            enabled: true,
            overdue_tasks: true,
            ..NotificationSettings::default()
        };
        let mut service = NotificationService::from_config(&config);
        service.begin_day(NaiveDate::from_ymd_opt(2026, 11, 5).unwrap());

        let report = report_with_items();
        let notifications = service.build(&report, &config);
        let mut notifier = RecordingNotifier::default();

        assert_eq!(
            service.dispatch(&notifications, &mut notifier).unwrap(),
            notifications.len()
        );

        service.begin_day(NaiveDate::from_ymd_opt(2026, 11, 6).unwrap());

        assert_eq!(
            service.dispatch(&notifications, &mut notifier).unwrap(),
            notifications.len()
        );
        assert_eq!(notifier.delivered.len(), notifications.len() * 2);
    }

    #[test]
    fn dispatch_propagates_delivery_errors() {
        let config = NotificationSettings {
            enabled: true,
            overdue_tasks: true,
            ..NotificationSettings::default()
        };
        let mut service = NotificationService::from_config(&config);
        let report = report_with_items();
        let notifications = service.build(&report, &config);

        let result = service.dispatch(&notifications, &mut FailingNotifier);

        assert_eq!(
            result,
            Err(NotificationError::Delivery("falha simulada".into()))
        );
    }

    #[test]
    fn dispatch_is_a_noop_when_disabled() {
        let config = NotificationSettings::default();
        let mut service = NotificationService::from_config(&config);
        let notifications = vec![Notification::new(
            NotificationKind::OverdueTask,
            "Atrasada",
            "corpo",
        )];
        let mut notifier = RecordingNotifier::default();

        let delivered = service
            .dispatch(&notifications, &mut notifier)
            .expect("dispatch deve funcionar");

        assert_eq!(delivered, 0);
        assert!(notifier.delivered.is_empty());
    }
}
