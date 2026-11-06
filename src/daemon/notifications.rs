use crate::config::NotificationSettings;
use crate::daemon::scheduler::CheckReport;
use crate::errors::AppResult;

/// Tipo de aviso gerado pelo daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    UpcomingTask,
    OverdueTask,
    UpcomingEvent,
}

/// Notificação pronta para ser entregue por um `Notifier`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub kind: NotificationKind,
    pub title: String,
    pub body: String,
}

/// Abstração de entrega de notificações.
///
/// A implementação concreta para notificações desktop será adicionada em uma
/// fase futura. Nesta fase apenas a interface existe, sem dependências de
/// sistema operacional.
pub trait Notifier {
    fn notify(&mut self, notification: &Notification) -> AppResult<()>;
}

/// Implementação nula usada como padrão seguro.
///
/// Ela permite que o daemon execute o fluxo completo de geração de
/// notificações sem efetivamente exibir nada na área de trabalho.
#[derive(Debug, Default)]
pub struct NoopNotifier;

impl Notifier for NoopNotifier {
    fn notify(&mut self, _notification: &Notification) -> AppResult<()> {
        Ok(())
    }
}

/// Serviço responsável por transformar verificações em notificações e
/// encaminhá-las para um `Notifier`.
#[derive(Debug, Clone)]
pub struct NotificationService {
    enabled: bool,
}

impl NotificationService {
    pub fn from_config(config: &NotificationSettings) -> Self {
        Self {
            enabled: config.enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Gera notificações respeitando as preferências do usuário.
    pub fn build(&self, report: &CheckReport, config: &NotificationSettings) -> Vec<Notification> {
        if !self.enabled || !config.enabled {
            return Vec::new();
        }

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
                notifications.push(Notification {
                    kind: NotificationKind::OverdueTask,
                    title: "Tarefa atrasada".to_string(),
                    body,
                });
            }
        }

        if config.upcoming_tasks {
            for task in &report.upcoming_tasks {
                let body = match task.due_date {
                    Some(due) => format!(
                        "A tarefa \"{}\" vence em {}.",
                        task.title,
                        due.format("%d/%m/%Y")
                    ),
                    None => format!("A tarefa \"{}\" está se aproximando.", task.title),
                };
                notifications.push(Notification {
                    kind: NotificationKind::UpcomingTask,
                    title: "Prazo próximo".to_string(),
                    body,
                });
            }
        }

        if config.upcoming_events {
            for event in &report.upcoming_events {
                let time = event
                    .start_time
                    .map(|time| time.format("%H:%M").to_string())
                    .unwrap_or_else(|| "dia inteiro".to_string());
                notifications.push(Notification {
                    kind: NotificationKind::UpcomingEvent,
                    title: "Evento próximo".to_string(),
                    body: format!(
                        "O evento \"{}\" começa em {} ({}).",
                        event.title,
                        event.date.format("%d/%m/%Y"),
                        time
                    ),
                });
            }
        }

        notifications
    }

    /// Encaminha notificações para o entregador informado.
    ///
    /// Retorna quantas notificações foram processadas.
    pub fn dispatch(
        &self,
        notifications: &[Notification],
        notifier: &mut dyn Notifier,
    ) -> AppResult<usize> {
        if !self.enabled {
            return Ok(0);
        }

        for notification in notifications {
            notifier.notify(notification)?;
        }
        Ok(notifications.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NotificationSettings;
    use crate::daemon::scheduler::CheckReport;
    use crate::models::task::Task;
    use chrono::NaiveDate;

    #[derive(Default)]
    struct RecordingNotifier {
        delivered: Vec<Notification>,
    }

    impl Notifier for RecordingNotifier {
        fn notify(&mut self, notification: &Notification) -> AppResult<()> {
            self.delivered.push(notification.clone());
            Ok(())
        }
    }

    fn report_with_items() -> CheckReport {
        let mut overdue = Task::new("Conta atrasada".into());
        overdue.due_date = NaiveDate::from_ymd_opt(2026, 11, 4);

        let mut upcoming = Task::new("Prova".into());
        upcoming.due_date = NaiveDate::from_ymd_opt(2026, 11, 5);

        CheckReport {
            checked_at: None,
            upcoming_tasks: vec![upcoming],
            overdue_tasks: vec![overdue],
            upcoming_events: Vec::new(),
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
        let service = NotificationService::from_config(&config);
        let report = report_with_items();
        let notifications = service.build(&report, &config);
        let mut notifier = RecordingNotifier::default();

        let delivered = service
            .dispatch(&notifications, &mut notifier)
            .expect("dispatch deve funcionar");

        assert_eq!(delivered, notifications.len());
        assert_eq!(notifier.delivered.len(), notifications.len());
    }
}
