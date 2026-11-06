//! Infraestrutura do daemon independente do Calma.
//!
//! O daemon compartilha o mesmo banco SQLite e a mesma configuração da TUI,
//! mas não depende do ciclo de vida do terminal. Nesta fase ele fornece
//! apenas a base de ciclo de vida, agendamento e abstração de notificações.

pub mod notifications;
pub mod scheduler;
pub mod service;
