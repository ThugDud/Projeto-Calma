//! Backend de notificações via Freedesktop D-Bus.
//!
//! KDE Plasma, GNOME, XFCE e vários outros ambientes implementam a
//! especificação `org.freedesktop.Notifications`. Este backend usa a API
//! bloqueante do `zbus`, com uma conexão reutilizada e tempo limite de
//! método, sem criar um runtime assíncrono próprio.

use super::notifier::{Notification, NotificationError, Notifier};
use std::collections::HashMap;
use std::time::Duration;
use zbus::blocking::{Connection, Proxy};

pub const NOTIFICATIONS_SERVICE: &str = "org.freedesktop.Notifications";
pub const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";
pub const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";

/// Nome exibido pelo Calma nas notificações.
pub const APP_NAME: &str = "Calma";
/// Ícone vazio: o servidor de notificações usa o ícone padrão do aplicativo.
pub const APP_ICON: &str = "";
/// Tempo padrão de exibição da notificação, em milissegundos.
pub const EXPIRE_TIMEOUT_MS: i32 = 8_000;
/// Tempo limite padrão das chamadas D-Bus.
pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(2);

/// Informações retornadas por `GetServerInformation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInformation {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub spec_version: String,
}

/// Dados de uma chamada `Notify`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyRequest<'a> {
    pub app_name: &'a str,
    pub replaces_id: u32,
    pub app_icon: &'a str,
    pub summary: &'a str,
    pub body: &'a str,
    pub actions: &'a [&'a str],
    pub expire_timeout: i32,
}

/// Transporte D-Bus mínimo, separado para permitir testes sem barramento real.
pub trait FreedesktopTransport: Send {
    fn server_information(&self) -> Result<ServerInformation, NotificationError>;
    fn notify(&self, request: &NotifyRequest<'_>) -> Result<u32, NotificationError>;
}

/// Transporte real usando uma conexão de sessão reutilizada.
pub struct ZbusTransport {
    proxy: Proxy<'static>,
}

impl ZbusTransport {
    pub fn connect(timeout: Duration) -> Result<Self, NotificationError> {
        let connection = zbus::blocking::connection::Builder::session()
            .map_err(|err| {
                NotificationError::SessionBusUnavailable(format!(
                    "não foi possível localizar o barramento da sessão: {}",
                    err
                ))
            })?
            .method_timeout(timeout)
            .build()
            .map_err(|err| {
                NotificationError::SessionBusUnavailable(format!(
                    "não foi possível conectar ao barramento da sessão: {}",
                    err
                ))
            })?;

        Self::from_connection(&connection)
    }

    fn from_connection(connection: &Connection) -> Result<Self, NotificationError> {
        let proxy = Proxy::new(
            connection,
            NOTIFICATIONS_SERVICE,
            NOTIFICATIONS_PATH,
            NOTIFICATIONS_INTERFACE,
        )
        .map_err(|err| {
            NotificationError::SessionBusUnavailable(format!(
                "não foi possível criar o proxy de notificações: {}",
                err
            ))
        })?;

        Ok(Self { proxy })
    }
}

impl FreedesktopTransport for ZbusTransport {
    fn server_information(&self) -> Result<ServerInformation, NotificationError> {
        let (name, vendor, version, spec_version): (String, String, String, String) = self
            .proxy
            .call("GetServerInformation", &())
            .map_err(map_zbus_error)?;

        Ok(ServerInformation {
            name,
            vendor,
            version,
            spec_version,
        })
    }

    fn notify(&self, request: &NotifyRequest<'_>) -> Result<u32, NotificationError> {
        let actions: Vec<&str> = request.actions.to_vec();
        let hints: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();

        let id: u32 = self
            .proxy
            .call(
                "Notify",
                &(
                    request.app_name,
                    request.replaces_id,
                    request.app_icon,
                    request.summary,
                    request.body,
                    &actions,
                    &hints,
                    request.expire_timeout,
                ),
            )
            .map_err(map_zbus_error)?;

        Ok(id)
    }
}

/// Backend Freedesktop genérico.
pub struct FreedesktopNotifier {
    transport: Box<dyn FreedesktopTransport>,
    app_name: String,
    app_icon: String,
    expire_timeout: i32,
}

impl FreedesktopNotifier {
    /// Cria o backend conectando ao barramento da sessão.
    pub fn connect(timeout: Duration) -> Result<Self, NotificationError> {
        let transport = ZbusTransport::connect(timeout)?;
        Ok(Self::with_transport(Box::new(transport)))
    }

    /// Cria o backend com um transporte injetado (usado em testes).
    pub fn with_transport(transport: Box<dyn FreedesktopTransport>) -> Self {
        Self {
            transport,
            app_name: APP_NAME.to_string(),
            app_icon: APP_ICON.to_string(),
            expire_timeout: EXPIRE_TIMEOUT_MS,
        }
    }
}

impl Notifier for FreedesktopNotifier {
    fn notify(&mut self, notification: &Notification) -> Result<(), NotificationError> {
        let request = NotifyRequest {
            app_name: &self.app_name,
            // Sem deduplicação nesta fase: cada aviso cria uma nova entrada.
            replaces_id: 0,
            app_icon: &self.app_icon,
            summary: &notification.title,
            body: &notification.body,
            actions: &[],
            expire_timeout: self.expire_timeout,
        };

        self.transport.notify(&request).map(|_| ())
    }

    fn is_available(&self) -> Result<bool, NotificationError> {
        self.transport.server_information().map(|_| true)
    }

    fn backend_label(&self) -> &'static str {
        "Freedesktop D-Bus"
    }
}

fn map_zbus_error(err: zbus::Error) -> NotificationError {
    match &err {
        zbus::Error::MethodError(name, detail, _) => {
            let name = name.as_str();
            if matches!(
                name,
                "org.freedesktop.DBus.Error.ServiceUnknown"
                    | "org.freedesktop.DBus.Error.NameHasNoOwner"
            ) {
                NotificationError::ServiceUnavailable(
                    detail
                        .clone()
                        .unwrap_or_else(|| "serviço não registrado no barramento".to_string()),
                )
            } else {
                NotificationError::Delivery(err.to_string())
            }
        }
        zbus::Error::InputOutput(io) if io.kind() == std::io::ErrorKind::TimedOut => {
            NotificationError::Timeout(err.to_string())
        }
        zbus::Error::Connection(io, _) => NotificationError::SessionBusUnavailable(io.to_string()),
        _ => NotificationError::Delivery(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::notifications::notifier::NotificationKind;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct FakeTransport {
        available: bool,
        fail_with: Option<NotificationError>,
        delivered: Arc<Mutex<Vec<(String, String, String)>>>,
    }

    impl FreedesktopTransport for FakeTransport {
        fn server_information(&self) -> Result<ServerInformation, NotificationError> {
            if let Some(err) = &self.fail_with {
                return Err(err.clone());
            }
            if self.available {
                Ok(ServerInformation {
                    name: "fake".into(),
                    vendor: "teste".into(),
                    version: "1".into(),
                    spec_version: "1.2".into(),
                })
            } else {
                Err(NotificationError::ServiceUnavailable(
                    "serviço ausente".into(),
                ))
            }
        }

        fn notify(&self, request: &NotifyRequest<'_>) -> Result<u32, NotificationError> {
            if let Some(err) = &self.fail_with {
                return Err(err.clone());
            }
            self.delivered.lock().unwrap().push((
                request.app_name.to_string(),
                request.summary.to_string(),
                request.body.to_string(),
            ));
            Ok(42)
        }
    }

    #[test]
    fn availability_reflects_transport() {
        let available = FreedesktopNotifier::with_transport(Box::new(FakeTransport {
            available: true,
            ..FakeTransport::default()
        }));
        assert_eq!(available.is_available(), Ok(true));

        let unavailable = FreedesktopNotifier::with_transport(Box::new(FakeTransport {
            available: false,
            ..FakeTransport::default()
        }));
        assert!(matches!(
            unavailable.is_available(),
            Err(NotificationError::ServiceUnavailable(_))
        ));
    }

    #[test]
    fn delivery_uses_calma_app_name_and_notification_content() {
        let delivered = Arc::new(Mutex::new(Vec::new()));
        let mut notifier = FreedesktopNotifier::with_transport(Box::new(FakeTransport {
            available: true,
            delivered: delivered.clone(),
            ..FakeTransport::default()
        }));

        notifier
            .notify(&Notification::new(
                NotificationKind::UpcomingTask,
                "Prazo próximo",
                "A tarefa \"Prova\" vence em 10/11/2026.",
            ))
            .expect("entrega deve funcionar");

        let delivered = delivered.lock().unwrap();
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0].0, APP_NAME);
        assert_eq!(delivered[0].1, "Prazo próximo");
        assert!(delivered[0].2.contains("Prova"));
    }

    #[test]
    fn delivery_errors_are_propagated() {
        let mut notifier = FreedesktopNotifier::with_transport(Box::new(FakeTransport {
            available: true,
            fail_with: Some(NotificationError::Delivery("falha simulada".into())),
            ..FakeTransport::default()
        }));

        let result = notifier.notify(&Notification::new(
            NotificationKind::OverdueTask,
            "Atrasada",
            "corpo",
        ));
        assert_eq!(
            result,
            Err(NotificationError::Delivery("falha simulada".into()))
        );
    }

    #[test]
    fn missing_session_bus_is_reported() {
        let error = NotificationError::SessionBusUnavailable("sem endereço".into());
        assert!(error.to_string().contains("D-Bus"));
    }

    #[test]
    fn availability_propagates_session_bus_failure() {
        let notifier = FreedesktopNotifier::with_transport(Box::new(FakeTransport {
            available: true,
            fail_with: Some(NotificationError::SessionBusUnavailable(
                "barramento ausente".into(),
            )),
            ..FakeTransport::default()
        }));

        assert!(matches!(
            notifier.is_available(),
            Err(NotificationError::SessionBusUnavailable(_))
        ));
    }

    #[test]
    #[ignore = "envia uma notificação real; execute manualmente em uma sessão gráfica"]
    fn manual_kde_notification_smoke() {
        let mut notifier = FreedesktopNotifier::connect(Duration::from_secs(2))
            .expect("deve conectar ao barramento da sessão");

        assert_eq!(notifier.is_available(), Ok(true));

        notifier
            .notify(&Notification::new(
                NotificationKind::UpcomingTask,
                "Calma — teste de integração",
                "Notificação de teste manual do Calma via Freedesktop D-Bus.",
            ))
            .expect("o serviço de notificações deve aceitar a entrega");
    }
}
