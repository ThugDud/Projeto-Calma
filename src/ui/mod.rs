pub mod calendar;
pub mod events;
pub mod header;
pub mod hierarchy;
pub mod layout;
pub mod modal;
pub mod overview;
pub mod settings;
pub mod shortcuts;
pub mod tasks;
pub mod theme;

use crate::app::App;
use crate::errors::AppResult;
use ratatui::{Terminal, backend::Backend};

/// Renderiza um frame completo da interface.
pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> AppResult<()> {
    terminal
        .draw(|f| {
            layout::render(f, app);
        })
        .map_err(|e| crate::errors::AppError::Terminal(e.to_string()))?;
    Ok(())
}
