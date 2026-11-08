//! Detecção do ambiente gráfico Linux e entrega nativa de notificações.
//!
//! Este módulo é independente da TUI e do daemon. Ele expõe apenas:
//!
//! - uma representação testável do ambiente da sessão;
//! - uma abstração de entrega (`Notifier`);
//! - backends concretos (Freedesktop D-Bus e um backend nulo);
//! - seleção de backend com diagnóstico.
//!
//! A detecção do desktop e a disponibilidade do serviço de notificações são
//! conceitos separados: o fato de o KDE Plasma estar em execução não garante
//! que `org.freedesktop.Notifications` esteja acessível.

pub mod detection;
pub mod environment;
pub mod notifications;
