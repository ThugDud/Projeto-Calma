//! Modelo compartilhado de notificações e abstração de entrega.

use chrono::NaiveDate;
use std::fmt;

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
    /// Identidade estável usada para evitar entregas repetidas.
    ///
    /// Notificações sem chave são sempre entregues; o daemon sempre gera
    /// uma chave para os avisos de tarefas e eventos.
    pub dedup_key: Option<String>,
    /// Data a partir da qual a chave de deduplicação pode ser descartada.
    pub dedup_expires_on: Option<NaiveDate>,
}

impl Notification {
    pub fn new(kind: NotificationKind, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            body: body.into(),
            dedup_key: None,
            dedup_expires_on: None,
        }
    }

    /// Define a identidade usada pelo serviço para deduplicar a entrega.
    ///
    /// A chave sem expiração permanece válida durante toda a execução.
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.dedup_key = Some(key.into());
        self
    }

    /// Define a identidade e a data-limite usada para limpar o estado.
    pub fn with_key_until(mut self, key: impl Into<String>, expires_on: NaiveDate) -> Self {
        self.dedup_key = Some(key.into());
        self.dedup_expires_on = Some(expires_on);
        self
    }
}

/// Falhas de detecção de serviço ou de entrega de notificações.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationError {
    /// O barramento D-Bus da sessão não está acessível.
    SessionBusUnavailable(String),
    /// O serviço de notificações não está registrado ou não respondeu.
    ServiceUnavailable(String),
    /// O serviço não respondeu dentro do tempo limite.
    Timeout(String),
    /// O serviço respondeu com erro ao entregar a notificação.
    Delivery(String),
    /// Backend ou operação não suportada.
    Unsupported(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationError::SessionBusUnavailable(msg) => {
                write!(f, "barramento D-Bus da sessão indisponível: {}", msg)
            }
            NotificationError::ServiceUnavailable(msg) => {
                write!(f, "serviço de notificações indisponível: {}", msg)
            }
            NotificationError::Timeout(msg) => {
                write!(
                    f,
                    "tempo limite ao falar com o serviço de notificações: {}",
                    msg
                )
            }
            NotificationError::Delivery(msg) => {
                write!(f, "falha ao entregar notificação: {}", msg)
            }
            NotificationError::Unsupported(msg) => {
                write!(f, "notificação não suportada: {}", msg)
            }
        }
    }
}

impl std::error::Error for NotificationError {}

/// Abstração de entrega de notificações.
///
/// O daemon depende apenas deste trait. Backends concretos (Freedesktop
/// D-Bus, KDE e nulo) ficam isolados no módulo `desktop::notifications`.
pub trait Notifier {
    fn notify(&mut self, notification: &Notification) -> Result<(), NotificationError>;

    /// Verifica se o serviço usado pelo backend está disponível.
    fn is_available(&self) -> Result<bool, NotificationError> {
        Ok(true)
    }

    /// Rótulo curto do backend, exibido em diagnósticos.
    fn backend_label(&self) -> &'static str {
        "Desconhecido"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_error_has_readable_messages() {
        let error = NotificationError::ServiceUnavailable("sem dono".into());
        assert!(error.to_string().contains("indisponível"));
    }
}
