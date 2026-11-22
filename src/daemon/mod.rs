//! Infraestrutura do daemon independente do Calma.
//!
//! O daemon compartilha o mesmo banco SQLite e a mesma configuração da TUI,
//! mas não depende do ciclo de vida do terminal. Ele mantém apenas o
//! ciclo de vida, o agendamento e o uso da abstração de notificações; a
//! detecção de ambiente e os backends concretos vivem em `desktop`.

pub mod notifications;
pub mod scheduler;
pub mod service;
