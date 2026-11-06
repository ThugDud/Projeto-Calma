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
    // Carrega configuração e inicializa o banco SQLite
    let config = Config::new();
    let conn = db::init(&config.database_path)?;

    // Configura o terminal em modo alternado com raw mode
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

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
