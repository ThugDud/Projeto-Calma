//! Backend KDE Plasma.
//!
//! O KDE implementa o protocolo padrão `org.freedesktop.Notifications`.
//! Para não duplicar o protocolo, este backend é um invólucro fino sobre o
//! backend Freedesktop, mudando apenas o rótulo de diagnóstico.

use super::freedesktop::FreedesktopNotifier;
use super::notifier::{Notification, NotificationError, Notifier};
use std::time::Duration;

pub struct KdeNotifier {
    inner: FreedesktopNotifier,
}

impl KdeNotifier {
    pub fn new(inner: FreedesktopNotifier) -> Self {
        Self { inner }
    }

    pub fn connect(timeout: Duration) -> Result<Self, NotificationError> {
        FreedesktopNotifier::connect(timeout).map(Self::new)
    }
}

impl Notifier for KdeNotifier {
    fn notify(&mut self, notification: &Notification) -> Result<(), NotificationError> {
        self.inner.notify(notification)
    }

    fn is_available(&self) -> Result<bool, NotificationError> {
        self.inner.is_available()
    }

    fn backend_label(&self) -> &'static str {
        "KDE / Freedesktop D-Bus"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::notifications::freedesktop::{
        FreedesktopTransport, NotifyRequest, ServerInformation,
    };
    use crate::desktop::notifications::notifier::NotificationKind;

    struct AvailableTransport;

    impl FreedesktopTransport for AvailableTransport {
        fn server_information(&self) -> Result<ServerInformation, NotificationError> {
            Ok(ServerInformation {
                name: "Plasma".into(),
                vendor: "KDE".into(),
                version: "6".into(),
                spec_version: "1.2".into(),
            })
        }

        fn notify(&self, _request: &NotifyRequest<'_>) -> Result<u32, NotificationError> {
            Ok(1)
        }
    }

    #[test]
    fn kde_wrapper_uses_freedesktop_protocol() {
        let mut notifier = KdeNotifier::new(FreedesktopNotifier::with_transport(Box::new(
            AvailableTransport,
        )));

        assert_eq!(notifier.is_available(), Ok(true));
        assert_eq!(notifier.backend_label(), "KDE / Freedesktop D-Bus");
        assert!(
            notifier
                .notify(&Notification::new(
                    NotificationKind::UpcomingEvent,
                    "Evento",
                    "corpo"
                ))
                .is_ok()
        );
    }
}
