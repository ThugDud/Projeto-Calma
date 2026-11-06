use crate::app::{App, InputMode, View};
use crate::ui::calendar;
use crate::ui::events;
use crate::ui::header;
use crate::ui::hierarchy;
use crate::ui::modal;
use crate::ui::overview;
use crate::ui::settings;
use crate::ui::shortcuts;
use crate::ui::tasks;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

/// Renderiza o layout completo da aplicação.
pub fn render(f: &mut Frame, app: &App) {
    let theme = &Theme::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(5), // cabeçalho
            Constraint::Min(1),    // conteúdo principal
            Constraint::Length(3), // barra de atalhos
        ])
        .split(f.area());

    header::render(f, chunks[0], theme, app.today, app);

    // Área de conteúdo principal baseada na visualização ativa
    match app.current_view {
        View::Calendario => {
            calendar::render(f, chunks[1], theme, app);
        }
        View::Tarefas => {
            tasks::render(f, chunks[1], theme, app);
        }
        View::Hierarquia => {
            hierarchy::render(f, chunks[1], theme, app);
        }
        View::VisaoGeral => {
            overview::render(f, chunks[1], theme, app);
        }
        View::Eventos => {
            events::render(f, chunks[1], theme, app);
        }
        View::Configuracoes => {
            settings::render(f, chunks[1], theme, app);
        }
    }

    shortcuts::render(f, chunks[2], theme, app);

    // Modais e sobreposições baseados no modo de entrada
    match &app.input_mode {
        InputMode::QuickAdd => {
            modal::render_quick_add(f, f.area(), theme, app);
        }
        InputMode::QuickAddEvent => {
            modal::render_quick_add_event(f, f.area(), theme, app);
        }
        InputMode::ConfirmDelete { target, title, .. } => {
            modal::render_confirm_delete(f, f.area(), theme, *target, title);
        }
        InputMode::EditTask(edit) => {
            modal::render_edit_task(f, f.area(), theme, app, edit);
        }
        InputMode::EditEvent(edit) => {
            modal::render_edit_event(f, f.area(), theme, app, edit);
        }
        InputMode::NewProject {
            parent_project_id,
            buffer,
        } => {
            modal::render_new_project(f, f.area(), theme, app, *parent_project_id, buffer);
        }
        InputMode::EditProject { buffer, .. } => {
            modal::render_edit_project(f, f.area(), theme, buffer);
        }
        InputMode::NewSubtask {
            parent_task_title,
            buffer,
            ..
        } => {
            modal::render_new_subtask(f, f.area(), theme, app, parent_task_title, buffer);
        }
        InputMode::Normal => {
            if app.show_help {
                modal::render_help_modal(f, f.area(), theme);
            }
        }
    }
}
