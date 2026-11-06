#![allow(dead_code)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_repeat_n)]

use crossterm::ExecutableCommand;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

mod app;
mod config;
mod daemon;
mod db;
mod errors;
mod input;
mod models;
mod services;
mod ui;

use app::App;
use config::Config;
use errors::AppResult;
use input::{handle_event, run_input_loop};
use ui::draw;

fn main() -> AppResult<()> {
    match std::env::args().nth(1).as_deref() {
        Some("daemon") | Some("--daemon") => run_daemon(),
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        _ => run_tui(),
    }
}

/// Executa a interface interativa padrão do Calma.
fn run_tui() -> AppResult<()> {
    // Carrega a configuração persistente e inicializa o banco SQLite
    let config = Config::load();
    let conn = db::init(&config.database_path)?;

    // Configura o terminal em modo alternado com raw mode
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::with_config(config);

    // Carrega dados persistentes reais do usuário (sem dados fictícios automáticos)
    app.reload(&conn)?;

    // Desenha a tela inicial
    draw(&mut terminal, &app)?;

    let app_result = run_input_loop(app, conn, |app, conn, event| {
        let continue_running = handle_event(app, conn, event)?;
        draw(&mut terminal, app)?;
        Ok(continue_running)
    });

    // Restaura o terminal ao sair
    let _ = io::stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();

    app_result?;
    Ok(())
}

/// Executa o daemon em primeiro plano.
///
/// O processo é independente da TUI e compartilha o mesmo banco SQLite e a
/// mesma configuração. Nesta fase não há integração com systemd, IPC ou
/// entrega real de notificações desktop.
fn run_daemon() -> AppResult<()> {
    let config = Config::load();

    if let Some(warning) = &config.load_warning {
        eprintln!("Aviso de configuração: {}", warning);
    }

    if !config.general.background_monitoring {
        println!(
            "Calma daemon: o monitoramento em segundo plano está desativado nas configurações."
        );
        println!("Ative a opção na tela de Configurações (tecla 6) da TUI.");
        return Ok(());
    }

    println!(
        "Calma daemon iniciado (intervalo: {}s, banco: {}).",
        config.general.poll_interval_secs,
        config.database_path.display()
    );

    let mut service = daemon::service::DaemonService::new(config)?;
    service.run()
}

fn print_help() {
    println!("Calma — gerenciador pessoal de tarefas e eventos");
    println!();
    println!("Uso:");
    println!("  calma           Inicia a interface TUI (padrão)");
    println!("  calma daemon    Inicia o daemon em primeiro plano");
    println!("  calma --help    Exibe esta ajuda");
    println!();
    println!(
        "Configuração: $XDG_CONFIG_HOME/calma/config.toml (padrão: ~/.config/calma/config.toml)"
    );
    println!("Dados:        $XDG_DATA_HOME/calma/calma.db (padrão: ~/.local/share/calma/calma.db)");
}
