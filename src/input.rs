use crate::app::{App, InputMode, View};
use crate::errors::AppResult;
use crate::models::task::TaskPriority;
use chrono::{Datelike, Duration};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use rusqlite::Connection;

/// Processa eventos de entrada de teclado.
///
/// Retorna `Ok(true)` se o aplicativo deve continuar, `Ok(false)` se deve sair.
pub fn handle_event(app: &mut App, conn: &Connection, event: Event) -> AppResult<bool> {
    match event {
        Event::Key(key) => handle_key(app, conn, key),
        Event::Resize(_, _) => Ok(true),
        _ => Ok(true),
    }
}

fn handle_key(app: &mut App, conn: &Connection, key: KeyEvent) -> AppResult<bool> {
    // 1. MODO: Cadastro Rápido de tarefa ou evento
    if matches!(
        app.input_mode,
        InputMode::QuickAdd | InputMode::QuickAddEvent
    ) {
        let is_event_mode = app.input_mode == InputMode::QuickAddEvent;
        match key.code {
            KeyCode::Enter => {
                let is_event =
                    is_event_mode || app.input_buffer.trim().to_lowercase().starts_with("event:");
                if is_event {
                    app.add_event_from_buffer(conn)?;
                } else {
                    app.add_task_from_buffer(conn)?;
                }
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.input_buffer.clear();
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        }
        return Ok(true);
    }

    // 2. MODO: Confirmação de Exclusão (Prioridade 2)
    if let InputMode::ConfirmDelete { .. } = app.input_mode {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                app.confirm_delete(conn)?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        return Ok(true);
    }

    // 3. MODO: Edição de Tarefa (Prioridade 3)
    if let InputMode::EditTask(ref mut edit) = app.input_mode {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                return Ok(true);
            }
            KeyCode::Tab => {
                edit.active_field = (edit.active_field + 1) % 4;
                return Ok(true);
            }
            KeyCode::BackTab => {
                edit.active_field = if edit.active_field == 0 {
                    3
                } else {
                    edit.active_field - 1
                };
                return Ok(true);
            }
            KeyCode::Enter => {
                app.save_edit_task(conn)?;
                return Ok(true);
            }
            _ => {
                match edit.active_field {
                    0 => match key.code {
                        KeyCode::Backspace => {
                            edit.title.pop();
                        }
                        KeyCode::Char(c) => {
                            edit.title.push(c);
                        }
                        _ => {}
                    },
                    1 => match key.code {
                        KeyCode::Backspace => {
                            edit.due_date_str.pop();
                        }
                        KeyCode::Char(c) => {
                            edit.due_date_str.push(c);
                        }
                        _ => {}
                    },
                    2 => match key.code {
                        KeyCode::Char('1') => edit.priority = TaskPriority::Baixa,
                        KeyCode::Char('2') => edit.priority = TaskPriority::Normal,
                        KeyCode::Char('3') => edit.priority = TaskPriority::Alta,
                        KeyCode::Char('4') => edit.priority = TaskPriority::Urgente,
                        KeyCode::Left | KeyCode::Down => {
                            edit.priority = match edit.priority {
                                TaskPriority::Urgente => TaskPriority::Alta,
                                TaskPriority::Alta => TaskPriority::Normal,
                                TaskPriority::Normal => TaskPriority::Baixa,
                                TaskPriority::Baixa => TaskPriority::Urgente,
                            };
                        }
                        KeyCode::Right | KeyCode::Up | KeyCode::Char(' ') => {
                            edit.priority = match edit.priority {
                                TaskPriority::Baixa => TaskPriority::Normal,
                                TaskPriority::Normal => TaskPriority::Alta,
                                TaskPriority::Alta => TaskPriority::Urgente,
                                TaskPriority::Urgente => TaskPriority::Baixa,
                            };
                        }
                        _ => {}
                    },
                    3 => match key.code {
                        KeyCode::Char('p')
                        | KeyCode::Char('P')
                        | KeyCode::Char(' ')
                        | KeyCode::Right => {
                            // Alterna projetos disponíveis: None -> proj1 -> proj2 -> ... -> None
                            let projs = &app.projects;
                            if projs.is_empty() {
                                edit.project_id = None;
                            } else {
                                match edit.project_id {
                                    None => edit.project_id = Some(projs[0].id),
                                    Some(curr_id) => {
                                        let curr_idx = projs.iter().position(|p| p.id == curr_id);
                                        if let Some(idx) = curr_idx {
                                            if idx + 1 < projs.len() {
                                                edit.project_id = Some(projs[idx + 1].id);
                                            } else {
                                                edit.project_id = None;
                                            }
                                        } else {
                                            edit.project_id = None;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
                return Ok(true);
            }
        }
    }

    // 3.1 MODO: Edição de Evento
    if let InputMode::EditEvent(ref mut edit) = app.input_mode {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                return Ok(true);
            }
            KeyCode::Tab => {
                edit.active_field = (edit.active_field + 1) % 3;
                return Ok(true);
            }
            KeyCode::BackTab => {
                edit.active_field = if edit.active_field == 0 {
                    2
                } else {
                    edit.active_field - 1
                };
                return Ok(true);
            }
            KeyCode::Enter => {
                app.save_edit_event(conn)?;
                return Ok(true);
            }
            _ => {
                match edit.active_field {
                    0 => match key.code {
                        KeyCode::Backspace => {
                            edit.title.pop();
                        }
                        KeyCode::Char(c) => {
                            edit.title.push(c);
                        }
                        _ => {}
                    },
                    1 => match key.code {
                        KeyCode::Backspace => {
                            edit.date_str.pop();
                        }
                        KeyCode::Char(c) => {
                            edit.date_str.push(c);
                        }
                        _ => {}
                    },
                    2 => match key.code {
                        KeyCode::Backspace => {
                            edit.time_str.pop();
                        }
                        KeyCode::Char(c) => {
                            edit.time_str.push(c);
                        }
                        _ => {}
                    },
                    _ => {}
                }
                return Ok(true);
            }
        }
    }

    // 4. MODO: Novo Projeto / Subprojeto (Prioridade 4)
    if let InputMode::NewProject { .. } = app.input_mode {
        match key.code {
            KeyCode::Enter => {
                app.save_new_project(conn)?;
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                if let InputMode::NewProject { ref mut buffer, .. } = app.input_mode {
                    buffer.pop();
                }
            }
            KeyCode::Char(c) => {
                if let InputMode::NewProject { ref mut buffer, .. } = app.input_mode {
                    buffer.push(c);
                }
            }
            _ => {}
        }
        return Ok(true);
    }

    // 5. MODO: Renomear Projeto
    if let InputMode::EditProject { .. } = app.input_mode {
        match key.code {
            KeyCode::Enter => {
                app.save_edit_project(conn)?;
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                if let InputMode::EditProject { ref mut buffer, .. } = app.input_mode {
                    buffer.pop();
                }
            }
            KeyCode::Char(c) => {
                if let InputMode::EditProject { ref mut buffer, .. } = app.input_mode {
                    buffer.push(c);
                }
            }
            _ => {}
        }
        return Ok(true);
    }

    // 6. MODO: Nova Subtarefa (Prioridade 4)
    if let InputMode::NewSubtask { .. } = app.input_mode {
        match key.code {
            KeyCode::Enter => {
                app.save_new_subtask(conn)?;
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                if let InputMode::NewSubtask { ref mut buffer, .. } = app.input_mode {
                    buffer.pop();
                }
            }
            KeyCode::Char(c) => {
                if let InputMode::NewSubtask { ref mut buffer, .. } = app.input_mode {
                    buffer.push(c);
                }
            }
            _ => {}
        }
        return Ok(true);
    }

    // Se a ajuda estiver aberta, apenas Esc, q ou ? fecha
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                app.show_help = false;
                return Ok(true);
            }
            _ => return Ok(true),
        }
    }

    // 7. Tela de configurações: navegação, alternância e edição numérica.
    if app.current_view == View::Configuracoes {
        return handle_settings_key(app, key);
    }

    // Atalhos globais no MODO NORMAL
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
            return Ok(false);
        }
        KeyCode::Char('?') | KeyCode::Char('/') => {
            app.show_help = true;
            return Ok(true);
        }
        KeyCode::Esc => {
            app.show_help = false;
            return Ok(true);
        }
        KeyCode::Char('1') => {
            app.current_view = View::Calendario;
            return Ok(true);
        }
        KeyCode::Char('2') => {
            app.current_view = View::Tarefas;
            return Ok(true);
        }
        KeyCode::Char('3') => {
            app.current_view = View::Hierarquia;
            return Ok(true);
        }
        KeyCode::Char('4') => {
            app.current_view = View::VisaoGeral;
            return Ok(true);
        }
        KeyCode::Char('5') => {
            app.current_view = View::Eventos;
            return Ok(true);
        }
        KeyCode::Char('6') => {
            app.open_settings();
            return Ok(true);
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            if app.current_view == View::Eventos {
                app.start_new_event();
            } else {
                app.input_mode = InputMode::QuickAdd;
                app.input_buffer.clear();
            }
            return Ok(true);
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.start_new_project();
            return Ok(true);
        }
        KeyCode::Char('e') => {
            app.start_edit();
            return Ok(true);
        }
        KeyCode::Char('E') => {
            app.start_new_event();
            return Ok(true);
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.start_confirm_delete();
            return Ok(true);
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.go_to_today();
            return Ok(true);
        }
        KeyCode::Char(' ') => {
            app.toggle_selected_task(conn)?;
            return Ok(true);
        }
        _ => {}
    }

    // Atalhos específicos por visualização
    match app.current_view {
        View::Calendario => handle_calendar_key(app, key),
        View::Tarefas => handle_tasks_key(app, key),
        View::Hierarquia => handle_hierarchy_key(app, key),
        View::VisaoGeral => handle_overview_key(app, key),
        View::Eventos => handle_eventos_key(app, key),
        View::Configuracoes => handle_settings_key(app, key),
    }
}

fn handle_settings_key(app: &mut App, key: KeyEvent) -> AppResult<bool> {
    // Edição numérica tem prioridade sobre os atalhos da tela.
    if app.settings.editing_value.is_some() {
        match key.code {
            KeyCode::Enter => app.settings_commit_edit(),
            KeyCode::Esc => app.settings_cancel_edit(),
            KeyCode::Backspace => app.settings_pop_char(),
            KeyCode::Char(c) => app.settings_push_char(c),
            _ => {}
        }
        return Ok(true);
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
            return Ok(false);
        }
        KeyCode::Char('?') | KeyCode::Char('/') => {
            app.show_help = true;
        }
        KeyCode::Esc => {
            app.close_settings();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.settings_select_previous();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.settings_select_next();
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            app.settings_activate_selected();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.refresh_desktop_diagnostics();
        }
        KeyCode::Char('1') => {
            app.current_view = View::Calendario;
        }
        KeyCode::Char('2') => {
            app.current_view = View::Tarefas;
        }
        KeyCode::Char('3') => {
            app.current_view = View::Hierarquia;
        }
        KeyCode::Char('4') => {
            app.current_view = View::VisaoGeral;
        }
        KeyCode::Char('5') => {
            app.current_view = View::Eventos;
        }
        KeyCode::Char('6') => {
            app.open_settings();
        }
        _ => {}
    }

    Ok(true)
}

fn handle_tasks_key(app: &mut App, key: KeyEvent) -> AppResult<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
            Ok(true)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            Ok(true)
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.current_view = View::Calendario;
            Ok(true)
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.current_view = View::Hierarquia;
            Ok(true)
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.current_view = View::VisaoGeral;
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_hierarchy_key(app: &mut App, key: KeyEvent) -> AppResult<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
            Ok(true)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            Ok(true)
        }
        KeyCode::Enter => {
            app.toggle_hierarchy_expand();
            Ok(true)
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.start_new_subtask();
            Ok(true)
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.current_view = View::Calendario;
            Ok(true)
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.current_view = View::Tarefas;
            Ok(true)
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.current_view = View::VisaoGeral;
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_overview_key(app: &mut App, key: KeyEvent) -> AppResult<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
            Ok(true)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            Ok(true)
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.current_view = View::Calendario;
            Ok(true)
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.current_view = View::Tarefas;
            Ok(true)
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.current_view = View::Hierarquia;
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_eventos_key(app: &mut App, key: KeyEvent) -> AppResult<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
            Ok(true)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            Ok(true)
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.current_view = View::Calendario;
            Ok(true)
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.current_view = View::Tarefas;
            Ok(true)
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.current_view = View::Hierarquia;
            Ok(true)
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.current_view = View::VisaoGeral;
            Ok(true)
        }
        _ => Ok(true),
    }
}

fn handle_calendar_key(app: &mut App, key: KeyEvent) -> AppResult<bool> {
    match key.code {
        KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Char('l') | KeyCode::Char('L') => {
            app.current_view = View::Tarefas;
            Ok(true)
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.current_view = View::Hierarquia;
            Ok(true)
        }
        KeyCode::Char('j') => {
            app.select_next();
            Ok(true)
        }
        KeyCode::Char('k') => {
            app.select_previous();
            Ok(true)
        }
        KeyCode::Left => {
            let current = app.selected_day.unwrap_or(app.today);
            let prev = current.pred_opt().unwrap_or(current);
            app.select_day(prev);
            if prev.month() != app.calendar_month || prev.year() != app.calendar_year {
                app.calendar_year = prev.year();
                app.calendar_month = prev.month();
            }
            Ok(true)
        }
        KeyCode::Right => {
            let current = app.selected_day.unwrap_or(app.today);
            let next = current.succ_opt().unwrap_or(current);
            app.select_day(next);
            if next.month() != app.calendar_month || next.year() != app.calendar_year {
                app.calendar_year = next.year();
                app.calendar_month = next.month();
            }
            Ok(true)
        }
        KeyCode::Up => {
            let current = app.selected_day.unwrap_or(app.today);
            let prev = current
                .checked_sub_signed(Duration::days(7))
                .unwrap_or(current);
            app.select_day(prev);
            if prev.month() != app.calendar_month || prev.year() != app.calendar_year {
                app.calendar_year = prev.year();
                app.calendar_month = prev.month();
            }
            Ok(true)
        }
        KeyCode::Down => {
            let current = app.selected_day.unwrap_or(app.today);
            let next = current
                .checked_add_signed(Duration::days(7))
                .unwrap_or(current);
            app.select_day(next);
            if next.month() != app.calendar_month || next.year() != app.calendar_year {
                app.calendar_year = next.year();
                app.calendar_month = next.month();
            }
            Ok(true)
        }
        KeyCode::PageUp => {
            app.previous_month();
            Ok(true)
        }
        KeyCode::PageDown => {
            app.next_month();
            Ok(true)
        }
        _ => Ok(true),
    }
}

/// Captura eventos de teclado do terminal e os repassa para o manipulador.
pub fn run_input_loop<F>(mut app: App, conn: Connection, mut handler: F) -> AppResult<()>
where
    F: FnMut(&mut App, &Connection, Event) -> AppResult<bool>,
{
    use crossterm::ExecutableCommand;
    use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};

    let result = (|| -> AppResult<()> {
        while app.running {
            // Atualiza a data do sistema em tempo de execução (Prioridade 6)
            app.update_current_date();

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        if !handler(&mut app, &conn, Event::Key(key))? {
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    })();

    let _ = std::io::stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();

    result
}
