//! Backends de notificação, seleção automática e diagnóstico de ambiente.

pub mod freedesktop;
pub mod kde;
pub mod noop;
pub mod notifier;

pub use freedesktop::FreedesktopNotifier;
pub use kde::KdeNotifier;
pub use noop::NoopNotifier;
pub use notifier::{NotificationError, Notifier};

use crate::config::Config;
use crate::desktop::detection::DesktopSession;
use std::time::Duration;

/// Backend efetivamente selecionado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Kde,
    Freedesktop,
    Noop,
}

impl BackendKind {
    pub fn label(self) -> &'static str {
        match self {
            BackendKind::Kde => "KDE / Freedesktop D-Bus",
            BackendKind::Freedesktop => "Freedesktop D-Bus",
            BackendKind::Noop => "Nenhum",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            BackendKind::Kde => "kde",
            BackendKind::Freedesktop => "freedesktop",
            BackendKind::Noop => "noop",
        }
    }
}

/// Preferência de backend lida da configuração.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendPreference {
    Auto,
    Kde,
    Freedesktop,
    Noop,
    Unknown(String),
}

impl BackendPreference {
    /// Interpreta o valor persistido em `[notifications.desktop] backend`.
    ///
    /// Valores desconhecidos não causam erro: eles são preservados para que a
    /// seleção possa emitir um diagnóstico e cair de volta em `auto`.
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "auto" => BackendPreference::Auto,
            "kde" | "plasma" | "kde-plasma" => BackendPreference::Kde,
            "freedesktop" | "dbus" | "freedesktop-dbus" => BackendPreference::Freedesktop,
            "noop" | "none" | "desativado" => BackendPreference::Noop,
            _ => BackendPreference::Unknown(value.trim().to_string()),
        }
    }

    /// Backend que seria usado antes de verificar o serviço.
    pub fn preview_backend(&self, session: &DesktopSession) -> BackendKind {
        match self {
            BackendPreference::Kde => BackendKind::Kde,
            BackendPreference::Freedesktop => BackendKind::Freedesktop,
            BackendPreference::Noop => BackendKind::Noop,
            BackendPreference::Auto | BackendPreference::Unknown(_) => {
                if session.environment.is_headless() {
                    BackendKind::Noop
                } else if session.environment.is_kde() {
                    BackendKind::Kde
                } else {
                    BackendKind::Freedesktop
                }
            }
        }
    }
}

/// Estado observado do serviço de notificações.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAvailability {
    Available,
    Unavailable,
    Unknown,
}

impl ServiceAvailability {
    pub fn label(self) -> &'static str {
        match self {
            ServiceAvailability::Available => "Disponível",
            ServiceAvailability::Unavailable => "Indisponível",
            ServiceAvailability::Unknown => "Desconhecido",
        }
    }
}

/// Fábrica de backends; permite injetar implementações falsas nos testes.
pub trait NotifierFactory {
    fn create_kde(&self) -> Result<Box<dyn Notifier>, NotificationError>;
    fn create_freedesktop(&self) -> Result<Box<dyn Notifier>, NotificationError>;
}

/// Fábrica de produção usando D-Bus.
#[derive(Debug, Clone, Copy)]
pub struct DesktopNotifierFactory {
    pub connection_timeout: Duration,
}

impl Default for DesktopNotifierFactory {
    fn default() -> Self {
        Self {
            connection_timeout: freedesktop::DEFAULT_CONNECTION_TIMEOUT,
        }
    }
}

impl NotifierFactory for DesktopNotifierFactory {
    fn create_kde(&self) -> Result<Box<dyn Notifier>, NotificationError> {
        KdeNotifier::connect(self.connection_timeout).map(|notifier| Box::new(notifier) as Box<_>)
    }

    fn create_freedesktop(&self) -> Result<Box<dyn Notifier>, NotificationError> {
        FreedesktopNotifier::connect(self.connection_timeout)
            .map(|notifier| Box::new(notifier) as Box<_>)
    }
}

/// Resultado da seleção de backend.
pub struct BackendSelection {
    pub notifier: Box<dyn Notifier>,
    pub backend: BackendKind,
    pub availability: ServiceAvailability,
    pub diagnostic: String,
}

/// Seleciona o backend mais adequado para a sessão e a preferência dadas.
///
/// `auto` tenta o backend KDE quando o desktop é KDE e, em seguida, o backend
/// Freedesktop genérico. Ambientes headless nunca tentam acessar D-Bus.
pub fn select_backend(
    session: &DesktopSession,
    preference: BackendPreference,
    factory: &dyn NotifierFactory,
) -> BackendSelection {
    if session.environment.is_headless() {
        return noop_selection(
            "Nenhum ambiente gráfico detectado; notificações desktop não estão disponíveis."
                .to_string(),
        );
    }

    let mut notes = Vec::new();
    let preference = match preference {
        BackendPreference::Unknown(raw) => {
            notes.push(format!(
                "Backend \"{}\" não reconhecido; usando seleção automática.",
                raw
            ));
            BackendPreference::Auto
        }
        other => other,
    };

    match preference {
        BackendPreference::Noop => {
            notes.push("Backend nulo selecionado; nenhuma notificação será exibida.".to_string());
            noop_selection(notes.join(" "))
        }
        BackendPreference::Kde => match attempt(factory, BackendKind::Kde) {
            Ok(notifier) => available_selection(notifier, BackendKind::Kde, notes),
            Err(err) => {
                notes.push(format!("{}: {}", BackendKind::Kde.label(), err));
                noop_selection(notes.join(" "))
            }
        },
        BackendPreference::Freedesktop => match attempt(factory, BackendKind::Freedesktop) {
            Ok(notifier) => available_selection(notifier, BackendKind::Freedesktop, notes),
            Err(err) => {
                notes.push(format!("{}: {}", BackendKind::Freedesktop.label(), err));
                noop_selection(notes.join(" "))
            }
        },
        BackendPreference::Auto => {
            let candidates: &[BackendKind] = if session.environment.is_kde() {
                &[BackendKind::Kde, BackendKind::Freedesktop]
            } else {
                &[BackendKind::Freedesktop]
            };

            for kind in candidates {
                match attempt(factory, *kind) {
                    Ok(notifier) => return available_selection(notifier, *kind, notes),
                    Err(err) => notes.push(format!("{}: {}", kind.label(), err)),
                }
            }

            notes.push("Nenhum backend de notificação disponível.".to_string());
            noop_selection(notes.join(" "))
        }
        BackendPreference::Unknown(_) => unreachable!("preferência desconhecida já foi tratada"),
    }
}

fn attempt(
    factory: &dyn NotifierFactory,
    kind: BackendKind,
) -> Result<Box<dyn Notifier>, NotificationError> {
    let notifier = match kind {
        BackendKind::Kde => factory.create_kde()?,
        BackendKind::Freedesktop => factory.create_freedesktop()?,
        BackendKind::Noop => return Ok(Box::new(NoopNotifier)),
    };

    match notifier.is_available() {
        Ok(true) => Ok(notifier),
        Ok(false) => Err(NotificationError::ServiceUnavailable(format!(
            "{} não está disponível",
            kind.label()
        ))),
        Err(err) => Err(err),
    }
}

fn available_selection(
    notifier: Box<dyn Notifier>,
    backend: BackendKind,
    notes: Vec<String>,
) -> BackendSelection {
    let mut detail = notes.join(" ");
    if !detail.is_empty() {
        detail.push(' ');
    }
    detail.push_str(&format!(
        "Serviço de notificações disponível via {}.",
        backend.label()
    ));

    BackendSelection {
        notifier,
        backend,
        availability: ServiceAvailability::Available,
        diagnostic: detail,
    }
}

fn noop_selection(diagnostic: String) -> BackendSelection {
    BackendSelection {
        notifier: Box::new(NoopNotifier),
        backend: BackendKind::Noop,
        availability: ServiceAvailability::Unavailable,
        diagnostic,
    }
}

/// Diagnóstico somente leitura exibido na tela de configurações.
#[derive(Debug, Clone)]
pub struct DesktopDiagnostics {
    pub session: DesktopSession,
    pub backend: BackendKind,
    pub availability: ServiceAvailability,
    pub delivery_enabled: bool,
    pub diagnostic: String,
}

impl DesktopDiagnostics {
    /// Executa detecção e verifica o serviço de notificações.
    pub fn detect(config: &Config) -> Self {
        Self::detect_with(
            config,
            DesktopSession::detect(),
            &DesktopNotifierFactory::default(),
        )
    }

    /// Versão injetável usada pelos testes.
    pub fn detect_with(
        config: &Config,
        session: DesktopSession,
        factory: &dyn NotifierFactory,
    ) -> Self {
        let preference = BackendPreference::parse(&config.notifications.desktop.backend);
        let selection = select_backend(&session, preference, factory);
        let delivery_enabled = config.notifications.enabled;

        let diagnostic = if delivery_enabled {
            selection.diagnostic.clone()
        } else {
            format!(
                "Notificações desativadas nas configurações. {}",
                selection.diagnostic
            )
        };

        Self {
            session,
            backend: selection.backend,
            availability: selection.availability,
            delivery_enabled,
            diagnostic,
        }
    }

    /// Detecção apenas do ambiente, sem consultar D-Bus.
    ///
    /// Usada para inicializar a TUI sem custo de conexão; a tela de
    /// configurações executa [`Self::detect`] ao abrir e ao pressionar `r`.
    pub fn detect_environment(config: &Config) -> Self {
        let session = DesktopSession::detect();
        let preference = BackendPreference::parse(&config.notifications.desktop.backend);
        let backend = preference.preview_backend(&session);

        let (availability, diagnostic) = if session.environment.is_headless() {
            (
                ServiceAvailability::Unavailable,
                "Nenhum ambiente gráfico detectado; notificações desktop não estão disponíveis."
                    .to_string(),
            )
        } else {
            (
                ServiceAvailability::Unknown,
                "Diagnóstico do serviço ainda não executado; pressione r para atualizar."
                    .to_string(),
            )
        };

        Self {
            session,
            backend,
            availability,
            delivery_enabled: config.notifications.enabled,
            diagnostic,
        }
    }

    pub fn desktop_label(&self) -> String {
        self.session.environment.label()
    }

    pub fn session_label(&self) -> &'static str {
        self.session.session_type.label()
    }

    pub fn backend_label(&self) -> &'static str {
        self.backend.label()
    }

    pub fn service_label(&self) -> &'static str {
        self.availability.label()
    }

    pub fn delivery_label(&self) -> &'static str {
        if !self.delivery_enabled {
            "Desativada (configuração)"
        } else if self.availability == ServiceAvailability::Available {
            "Ativada"
        } else {
            "Ativada, mas sem backend disponível"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, NotificationSettings};
    use crate::desktop::detection::{DesktopEnvironment, SessionType};
    use crate::desktop::notifications::notifier::Notification;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Behavior {
        Available,
        Unavailable,
        Error,
    }

    struct FakeNotifier {
        behavior: Behavior,
    }

    impl Notifier for FakeNotifier {
        fn notify(&mut self, _notification: &Notification) -> Result<(), NotificationError> {
            Ok(())
        }

        fn is_available(&self) -> Result<bool, NotificationError> {
            match self.behavior {
                Behavior::Available => Ok(true),
                Behavior::Unavailable => Ok(false),
                Behavior::Error => Err(NotificationError::ServiceUnavailable(
                    "erro simulado".into(),
                )),
            }
        }

        fn backend_label(&self) -> &'static str {
            "fake"
        }
    }

    struct FakeFactory {
        kde: Behavior,
        freedesktop: Behavior,
    }

    impl FakeFactory {
        fn new(kde: Behavior, freedesktop: Behavior) -> Self {
            Self { kde, freedesktop }
        }

        fn all_available() -> Self {
            Self::new(Behavior::Available, Behavior::Available)
        }
    }

    impl NotifierFactory for FakeFactory {
        fn create_kde(&self) -> Result<Box<dyn Notifier>, NotificationError> {
            fake_notifier(self.kde)
        }

        fn create_freedesktop(&self) -> Result<Box<dyn Notifier>, NotificationError> {
            fake_notifier(self.freedesktop)
        }
    }

    fn fake_notifier(behavior: Behavior) -> Result<Box<dyn Notifier>, NotificationError> {
        match behavior {
            Behavior::Error => Err(NotificationError::ServiceUnavailable(
                "erro simulado".into(),
            )),
            other => Ok(Box::new(FakeNotifier { behavior: other })),
        }
    }

    fn session(environment: DesktopEnvironment) -> DesktopSession {
        DesktopSession {
            environment,
            session_type: SessionType::Wayland,
            raw_desktop: None,
        }
    }

    #[test]
    fn parses_backend_preferences() {
        assert_eq!(BackendPreference::parse("auto"), BackendPreference::Auto);
        assert_eq!(BackendPreference::parse(" KDE "), BackendPreference::Kde);
        assert_eq!(
            BackendPreference::parse("freedesktop"),
            BackendPreference::Freedesktop
        );
        assert_eq!(BackendPreference::parse("none"), BackendPreference::Noop);
        assert_eq!(
            BackendPreference::parse("telepathy"),
            BackendPreference::Unknown("telepathy".into())
        );
    }

    #[test]
    fn auto_selects_kde_on_kde_plasma() {
        let selection = select_backend(
            &session(DesktopEnvironment::KdePlasma),
            BackendPreference::Auto,
            &FakeFactory::all_available(),
        );

        assert_eq!(selection.backend, BackendKind::Kde);
        assert_eq!(selection.availability, ServiceAvailability::Available);
        assert!(selection.diagnostic.contains("disponível"));
    }

    #[test]
    fn auto_selects_freedesktop_for_gnome() {
        let selection = select_backend(
            &session(DesktopEnvironment::Gnome),
            BackendPreference::Auto,
            &FakeFactory::all_available(),
        );

        assert_eq!(selection.backend, BackendKind::Freedesktop);
        assert_eq!(selection.availability, ServiceAvailability::Available);
    }

    #[test]
    fn auto_falls_back_to_freedesktop_when_kde_is_unavailable() {
        let selection = select_backend(
            &session(DesktopEnvironment::KdePlasma),
            BackendPreference::Auto,
            &FakeFactory::new(Behavior::Unavailable, Behavior::Available),
        );

        assert_eq!(selection.backend, BackendKind::Freedesktop);
        assert_eq!(selection.availability, ServiceAvailability::Available);
        assert!(selection.diagnostic.contains("KDE"));
    }

    #[test]
    fn auto_falls_back_to_noop_when_all_backends_fail() {
        let selection = select_backend(
            &session(DesktopEnvironment::KdePlasma),
            BackendPreference::Auto,
            &FakeFactory::new(Behavior::Error, Behavior::Unavailable),
        );

        assert_eq!(selection.backend, BackendKind::Noop);
        assert_eq!(selection.availability, ServiceAvailability::Unavailable);
        assert!(selection.diagnostic.contains("Nenhum backend"));
    }

    #[test]
    fn headless_never_uses_a_graphical_backend() {
        let selection = select_backend(
            &session(DesktopEnvironment::Headless),
            BackendPreference::Auto,
            &FakeFactory::all_available(),
        );

        assert_eq!(selection.backend, BackendKind::Noop);
        assert_eq!(selection.availability, ServiceAvailability::Unavailable);
        assert!(selection.diagnostic.contains("Nenhum ambiente gráfico"));
    }

    #[test]
    fn explicit_kde_unavailable_falls_back_to_noop() {
        let selection = select_backend(
            &session(DesktopEnvironment::KdePlasma),
            BackendPreference::Kde,
            &FakeFactory::new(Behavior::Unavailable, Behavior::Available),
        );

        assert_eq!(selection.backend, BackendKind::Noop);
        assert!(selection.diagnostic.contains("KDE"));
    }

    #[test]
    fn invalid_backend_falls_back_to_auto() {
        let selection = select_backend(
            &session(DesktopEnvironment::KdePlasma),
            BackendPreference::Unknown("telepathy".into()),
            &FakeFactory::all_available(),
        );

        assert_eq!(selection.backend, BackendKind::Kde);
        assert!(selection.diagnostic.contains("não reconhecido"));
    }

    #[test]
    fn explicit_noop_is_honored() {
        let selection = select_backend(
            &session(DesktopEnvironment::KdePlasma),
            BackendPreference::Noop,
            &FakeFactory::all_available(),
        );

        assert_eq!(selection.backend, BackendKind::Noop);
        assert!(selection.diagnostic.contains("Backend nulo"));
    }

    #[test]
    fn diagnostics_distinguish_delivery_from_service_availability() {
        let config = Config {
            notifications: NotificationSettings {
                enabled: false,
                ..NotificationSettings::default()
            },
            ..Config::default()
        };
        let diagnostics = DesktopDiagnostics::detect_with(
            &config,
            session(DesktopEnvironment::KdePlasma),
            &FakeFactory::all_available(),
        );

        assert_eq!(diagnostics.desktop_label(), "KDE Plasma");
        assert_eq!(diagnostics.session_label(), "Wayland");
        assert_eq!(diagnostics.backend_label(), "KDE / Freedesktop D-Bus");
        assert_eq!(diagnostics.service_label(), "Disponível");
        assert_eq!(diagnostics.delivery_label(), "Desativada (configuração)");
        assert!(diagnostics.diagnostic.contains("desativadas"));
    }

    #[test]
    fn diagnostics_report_missing_service_without_claiming_success() {
        let config = Config {
            notifications: NotificationSettings {
                enabled: true,
                ..NotificationSettings::default()
            },
            ..Config::default()
        };
        let diagnostics = DesktopDiagnostics::detect_with(
            &config,
            session(DesktopEnvironment::Gnome),
            &FakeFactory::new(Behavior::Unavailable, Behavior::Unavailable),
        );

        assert_eq!(diagnostics.service_label(), "Indisponível");
        assert_eq!(
            diagnostics.delivery_label(),
            "Ativada, mas sem backend disponível"
        );
        assert!(diagnostics.diagnostic.contains("Nenhum backend"));
    }

    #[test]
    fn diagnostics_report_headless_session() {
        let config = Config::default();
        let diagnostics = DesktopDiagnostics::detect_with(
            &config,
            session(DesktopEnvironment::Headless),
            &FakeFactory::all_available(),
        );

        assert_eq!(diagnostics.desktop_label(), "Nenhum");
        assert_eq!(diagnostics.backend_label(), "Nenhum");
        assert_eq!(diagnostics.service_label(), "Indisponível");
    }

    #[test]
    fn environment_only_diagnostics_do_not_claim_service_state() {
        let mut session = session(DesktopEnvironment::KdePlasma);
        session.session_type = SessionType::X11;
        let diagnostics = DesktopDiagnostics {
            session,
            backend: BackendKind::Kde,
            availability: ServiceAvailability::Unknown,
            delivery_enabled: false,
            diagnostic: "pendente".into(),
        };

        assert_eq!(diagnostics.session_label(), "X11");
        assert_eq!(diagnostics.service_label(), "Desconhecido");
        assert_eq!(diagnostics.delivery_label(), "Desativada (configuração)");
    }
}
