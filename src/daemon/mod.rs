//! Infraestrutura do daemon independente do Calma.
//!
//! O daemon compartilha o mesmo banco SQLite e a mesma configuração da TUI,
//! mas não depende do ciclo de vida do terminal. Ele mantém o ciclo de vida,
//! o agendamento, o uso da abstração de notificações e o ícone de bandeja; a
//! detecção de ambiente, os backends concretos e a abertura da TUI vivem em
//! `desktop`.

pub mod notifications;
pub mod scheduler;
pub mod service;
