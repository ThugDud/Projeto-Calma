//! Backend nulo usado quando não há serviço de notificações disponível.

use super::notifier::{Notification, NotificationError, Notifier};

/// Implementação nula, segura e sem efeitos colaterais.
///
/// Ela permite que o daemon continue executando quando nenhum backend
/// gráfico está disponível.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopNotifier;

impl Notifier for NoopNotifier {
    fn notify(&mut self, _notification: &Notification) -> Result<(), NotificationError> {
        Ok(())
    }

    fn is_available(&self) -> Result<bool, NotificationError> {
        Ok(false)
    }

    fn backend_label(&self) -> &'static str {
        "Nenhum"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::notifications::notifier::NotificationKind;

    #[test]
    fn noop_never_delivers_but_never_fails() {
        let mut notifier = NoopNotifier;
        let notification = Notification::new(NotificationKind::OverdueTask, "t", "b");

        assert!(notifier.notify(&notification).is_ok());
        assert_eq!(notifier.is_available(), Ok(false));
        assert_eq!(notifier.backend_label(), "Nenhum");
    }
}
