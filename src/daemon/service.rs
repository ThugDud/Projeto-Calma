use crate::config::Config;
use crate::daemon::notifications::{NoopNotifier, NotificationService, Notifier};
use crate::daemon::scheduler::{CheckReport, Scheduler};
use crate::db;
use crate::errors::AppResult;
use rusqlite::Connection;
use std::time::{Duration, Instant};

/// Serviço principal do daemon.
///
/// O serviço abre o mesmo banco SQLite usado pela TUI, cria seu próprio
/// agendador e permanece independente de qualquer estado de terminal. A
/// execução é síncrona de propósito: não há necessidade de um runtime
/// assíncrono nesta fase.
pub struct DaemonService {
    config: Config,
    conn: Connection,
    scheduler: Scheduler,
    notifications: NotificationService,
    notifier: Box<dyn Notifier>,
    running: bool,
    last_report: Option<CheckReport>,
}

impl DaemonService {
    /// Inicializa o daemon a partir da configuração centralizada.
    pub fn new(config: Config) -> AppResult<Self> {
        let conn = db::init(&config.database_path)?;
        let scheduler = Scheduler::from_config(&config);
        let notifications = NotificationService::from_config(&config.notifications);

        Ok(Self {
            config,
            conn,
            scheduler,
            notifications,
            notifier: Box::new(NoopNotifier),
            running: false,
            last_report: None,
        })
    }

    /// Substitui o entregador de notificações.
    ///
    /// Útil para testes e para a futura implementação de notificações
    /// desktop, sem alterar a lógica do daemon.
    pub fn set_notifier(&mut self, notifier: Box<dyn Notifier>) {
        self.notifier = notifier;
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn last_report(&self) -> Option<&CheckReport> {
        self.last_report.as_ref()
    }

    /// Executa uma verificação única (sem dormir).
    ///
    /// Retorna um relatório vazio quando o monitoramento em segundo plano
    /// está desabilitado na configuração.
    pub fn tick(&mut self) -> AppResult<CheckReport> {
        if !self.config.general.background_monitoring {
            let report = CheckReport::default();
            self.last_report = Some(report.clone());
            return Ok(report);
        }

        let now = chrono::Local::now().naive_local();
        let report = self.scheduler.check(&self.conn, &self.config, now)?;
        let notifications = self
            .notifications
            .build(&report, &self.config.notifications);
        self.notifications
            .dispatch(&notifications, self.notifier.as_mut())?;

        self.scheduler.mark_checked(Instant::now());
        self.last_report = Some(report.clone());
        Ok(report)
    }

    /// Inicia o ciclo de vida do daemon e bloqueia até um pedido de parada.
    ///
    /// Se o monitoramento estiver desabilitado, o serviço encerra de forma
    /// limpa imediatamente. A TUI nunca inicia este loop automaticamente.
    pub fn run(&mut self) -> AppResult<()> {
        if !self.config.general.background_monitoring {
            self.shutdown();
            return Ok(());
        }

        self.running = true;
        while self.running {
            if self.scheduler.should_check(Instant::now()) {
                self.tick()?;
            }
            std::thread::sleep(self.poll_delay());
        }

        self.shutdown();
        Ok(())
    }

    /// Solicita o encerramento gracioso do loop principal.
    pub fn request_shutdown(&mut self) {
        self.running = false;
    }

    /// Encerra o serviço e libera o estado de execução.
    pub fn shutdown(&mut self) {
        self.running = false;
    }

    /// Intervalo de espera entre iterações do loop.
    ///
    /// Limitado a, no máximo, um segundo para que `request_shutdown` possa
    /// ser observado com razoável responsividade quando o intervalo for
    /// grande.
    fn poll_delay(&self) -> Duration {
        self.scheduler.interval().min(Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GeneralSettings, NotificationSettings};
    use crate::daemon::notifications::{Notification, Notifier};
    use crate::errors::AppResult;
    use crate::models::task::Task;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone, Default)]
    struct CountingNotifier {
        count: Arc<AtomicUsize>,
    }

    impl Notifier for CountingNotifier {
        fn notify(&mut self, _notification: &Notification) -> AppResult<()> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("relógio do sistema")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "calma-daemon-test-{}-{}-{}.db",
            std::process::id(),
            nanos,
            name
        ))
    }

    #[test]
    fn tick_is_a_noop_when_monitoring_is_disabled() {
        let path = temp_db_path("disabled");
        let config = Config {
            database_path: path.clone(),
            ..Config::default()
        };

        let mut service = DaemonService::new(config).expect("daemon deve inicializar");
        let report = service.tick().expect("tick deve funcionar");

        assert!(report.is_empty());
        assert!(!service.is_running());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn lifecycle_can_be_initialized_and_shut_down() {
        let path = temp_db_path("lifecycle");
        let config = Config {
            database_path: path.clone(),
            general: GeneralSettings {
                background_monitoring: true,
                ..GeneralSettings::default()
            },
            ..Config::default()
        };

        let mut service = DaemonService::new(config).expect("daemon deve inicializar");
        let _ = service.tick().expect("tick deve funcionar");
        service.request_shutdown();
        assert!(!service.is_running());
        service.shutdown();

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn tick_dispatches_generated_notifications() {
        let path = temp_db_path("dispatch");
        let config = Config {
            database_path: path.clone(),
            general: GeneralSettings {
                background_monitoring: true,
                ..GeneralSettings::default()
            },
            notifications: NotificationSettings {
                enabled: true,
                upcoming_tasks: true,
                overdue_tasks: false,
                upcoming_events: false,
                ..NotificationSettings::default()
            },
            ..Config::default()
        };

        let mut service = DaemonService::new(config).expect("daemon deve inicializar");
        let today = chrono::Local::now().date_naive();
        let mut task = Task::new("Tarefa de hoje".into());
        task.due_date = Some(today);
        db::insert_task(&service.conn, &task).expect("tarefa deve ser inserida");

        let counting = CountingNotifier::default();
        let delivered = counting.count.clone();
        service.set_notifier(Box::new(counting));
        let report = service.tick().expect("tick deve funcionar");

        assert_eq!(report.upcoming_tasks.len(), 1);
        assert!(service.last_report().is_some());
        assert_eq!(delivered.load(Ordering::SeqCst), 1);

        let _ = std::fs::remove_file(path);
    }
}
