use chrono::{Datelike, NaiveDate};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use crate::app::App;
use crate::models::task::TaskStatus;
use crate::services::calendar::{DIAS_SEMANA, MESES, month_matrix};
use crate::ui::theme::Theme;

const COL_WIDTH: u16 = 6;

/// Renderiza o calendário mensal e o painel contextual do dia selecionado.
pub fn render(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(46), Constraint::Min(20)])
        .split(area);

    let cal_area = chunks[0];
    let side_area = chunks[1];

    let year = app.calendar_year;
    let month = app.calendar_month;
    let today = app.today;
    let selected = app.selected_day;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style())
        .title(format!(
            " {} {} (PgUp/PgDn: Mês) ",
            MESES[(month - 1) as usize],
            year
        ));

    let inner = block.inner(cal_area);
    f.render_widget(block, cal_area);

    let widths = vec![Constraint::Length(COL_WIDTH); 7];

    let header_cells: Vec<Span> = DIAS_SEMANA
        .iter()
        .map(|dia| {
            Span::styled(
                format!("{:^width$}", dia, width = COL_WIDTH as usize - 1),
                Style::default()
                    .fg(theme.muted_fg)
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();
    let header_row = Row::new(header_cells).height(1);

    let matrix = month_matrix(year, month);
    let mut rows: Vec<Row> = Vec::new();
    for week in &matrix {
        let cells: Vec<Span> = week
            .iter()
            .map(|day_opt| cell(day_opt, today, selected, theme, app))
            .collect();
        rows.push(Row::new(cells).height(1));
    }

    let table = Table::new(rows, widths)
        .header(header_row)
        .column_spacing(0);

    let tw = inner.width.min(7 * COL_WIDTH);
    let th = inner.height.saturating_sub(2);
    f.render_widget(
        table,
        Rect {
            x: inner.x,
            y: inner.y,
            width: tw,
            height: th,
        },
    );

    let ly = inner.y.saturating_add(th);
    if ly < cal_area.bottom() {
        let legend = format!(
            "Hoje: {:02}/{:02} | Sel: {}",
            today.day(),
            today.month(),
            selected
                .map(|d| format!("{:02}/{:02}", d.day(), d.month()))
                .unwrap_or_else(|| "-".into())
        );
        f.render_widget(
            Paragraph::new(legend)
                .style(theme.muted_style())
                .alignment(ratatui::layout::Alignment::Center),
            Rect {
                x: inner.x,
                y: ly,
                width: inner.width,
                height: 1,
            },
        );
    }

    if ly.saturating_add(1) < cal_area.bottom() {
        let markers = Line::from(vec![
            Span::styled("• tarefas ", Style::default().fg(theme.due_today_fg)),
            Span::styled("◆ eventos ", Style::default().fg(theme.event_fg)),
            Span::styled("◉ ambos ", Style::default().fg(theme.task_event_fg)),
            Span::styled("! atrasadas", Style::default().fg(theme.overdue_fg)),
        ]);
        f.render_widget(
            Paragraph::new(markers).alignment(ratatui::layout::Alignment::Center),
            Rect {
                x: inner.x,
                y: ly + 1,
                width: inner.width,
                height: 1,
            },
        );
    }

    // Painel lateral: Tarefas e eventos do dia selecionado
    let sel_day = selected.unwrap_or(today);
    let day_tasks = app.tasks_for_day(sel_day);

    let day_events: Vec<_> = app.events.iter().filter(|e| e.date == sel_day).collect();

    let side_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style())
        .title(format!(
            " Compromissos do Dia {:02}/{:02}/{} ",
            sel_day.day(),
            sel_day.month(),
            sel_day.year()
        ));

    let side_inner = side_block.inner(side_area);
    f.render_widget(side_block, side_area);

    let mut side_lines = Vec::new();

    if !day_events.is_empty() {
        side_lines.push(Line::from(Span::styled(
            "Eventos:",
            Style::default().bold().fg(Color::Rgb(0x8f, 0xbc, 0xbb)),
        )));
        for ev in &day_events {
            let time_str = ev
                .start_time
                .map(|t| t.format("%H:%M ").to_string())
                .unwrap_or_default();
            side_lines.push(Line::from(vec![Span::styled(
                format!("  • {}{}", time_str, ev.title),
                Style::default().fg(theme.fg),
            )]));
        }
        side_lines.push(Line::from(""));
    }

    side_lines.push(Line::from(Span::styled(
        format!("Tarefas do dia ({}):", day_tasks.len()),
        Style::default().bold().fg(theme.fg),
    )));

    if day_tasks.is_empty() {
        side_lines.push(Line::from(Span::styled(
            "  (Nenhuma tarefa nesta data)",
            theme.muted_style(),
        )));
    } else {
        for (idx, t) in day_tasks.iter().enumerate() {
            let is_selected = idx == app.calendar_selected_task_idx;
            let status = app.task_status_on(t, sel_day);
            let (icon, color) = match status {
                TaskStatus::Concluida => ("[✓]", theme.completed_fg),
                TaskStatus::EmAndamento => ("[~]", Color::Rgb(0xeb, 0xcb, 0x8b)),
                TaskStatus::Pendente => ("[ ]", theme.fg),
            };
            let title_style = if is_selected {
                Style::default()
                    .bg(theme.selected_bg)
                    .fg(theme.selected_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };
            let time_prefix = t
                .scheduled_time
                .map(|time| format!("{} ", time.format("%H:%M")))
                .unwrap_or_default();
            let repeat_prefix = if t.is_recurring() { "↻ " } else { "" };
            side_lines.push(Line::from(vec![
                Span::styled(
                    if is_selected { "  > " } else { "    " },
                    Style::default().fg(theme.accent),
                ),
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(
                    format!("{}{}{}", repeat_prefix, time_prefix, t.title),
                    title_style,
                ),
            ]));
        }
    }

    let side_p = Paragraph::new(Text::from(side_lines));
    f.render_widget(side_p, side_inner);
}

/// Marcadores visuais de um dia no calendário.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayMarkers {
    /// Há tarefa não recorrente pendente na data.
    pub has_tasks: bool,
    /// Há evento na data.
    pub has_events: bool,
    /// Há tarefa não recorrente atrasada na data.
    pub has_overdue: bool,
}

/// Calcula o que deve ser destacado em um dia do calendário.
///
/// Tarefas recorrentes são intencionalmente ignoradas para não marcar
/// todos os dias do mês; elas continuam visíveis no painel do dia.
pub fn day_markers(app: &App, date: NaiveDate, today: NaiveDate) -> DayMarkers {
    let has_tasks = app.tasks.iter().any(|t| {
        !t.is_recurring()
            && t.status != TaskStatus::Concluida
            && (t.due_date == Some(date) || t.scheduled_date == Some(date))
    });
    let has_events = app.events.iter().any(|e| e.date == date);
    let has_overdue = app.tasks.iter().any(|t| {
        !t.is_recurring()
            && t.due_date == Some(date)
            && date < today
            && t.status != TaskStatus::Concluida
    });

    DayMarkers {
        has_tasks,
        has_events,
        has_overdue,
    }
}

fn cell(
    d: &Option<NaiveDate>,
    today: NaiveDate,
    sel: Option<NaiveDate>,
    theme: &Theme,
    app: &App,
) -> Span<'static> {
    match d {
        Some(date) => {
            let is_t = *date == today;
            let is_s = sel == Some(*date);

            let DayMarkers {
                has_tasks,
                has_events,
                has_overdue,
            } = day_markers(app, *date, today);

            let marker = if has_overdue {
                "!"
            } else if has_tasks && has_events {
                "◉"
            } else if has_tasks {
                "•"
            } else if has_events {
                "◆"
            } else {
                " "
            };

            let text = format!("{:>2}{:<2}", date.day(), marker);

            let st = if is_s {
                Style::default()
                    .bg(theme.selected_bg)
                    .fg(theme.selected_fg)
                    .add_modifier(Modifier::BOLD)
            } else if is_t {
                Style::default()
                    .bg(theme.today_bg)
                    .fg(theme.today_fg)
                    .add_modifier(Modifier::BOLD)
            } else if has_overdue {
                Style::default()
                    .fg(theme.overdue_fg)
                    .add_modifier(Modifier::BOLD)
            } else if has_tasks && has_events {
                Style::default()
                    .fg(theme.task_event_fg)
                    .add_modifier(Modifier::BOLD)
            } else if has_tasks {
                Style::default().fg(Color::Rgb(0xeb, 0xcb, 0x8b))
            } else if has_events {
                Style::default().fg(theme.event_fg)
            } else {
                Style::default().fg(theme.fg)
            };
            Span::styled(text, st)
        }
        None => Span::styled("    ", Style::default().fg(theme.muted_fg)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::event::Event;
    use crate::models::task::{Task, TaskRecurrence};
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn recurring_tasks_do_not_mark_calendar_days() {
        let mut app = App::new();
        let mut task = Task::new("Academia".into());
        task.recurrence = TaskRecurrence::Daily;
        app.tasks.push(task);

        let today = date(2026, 11, 11);
        let markers = day_markers(&app, today, today);
        assert!(!markers.has_tasks);
        assert!(!markers.has_events);
        assert!(!markers.has_overdue);
    }

    #[test]
    fn non_recurring_pending_task_marks_day_but_completed_does_not() {
        let mut app = App::new();
        let mut pending = Task::new("Prova".into());
        pending.due_date = Some(date(2026, 11, 12));
        app.tasks.push(pending);

        let today = date(2026, 11, 11);
        assert!(day_markers(&app, date(2026, 11, 12), today).has_tasks);

        app.tasks[0].status = crate::models::task::TaskStatus::Concluida;
        assert!(!day_markers(&app, date(2026, 11, 12), today).has_tasks);
    }

    #[test]
    fn events_and_combined_days_are_detected() {
        let mut app = App::new();
        let mut task = Task::new("Consulta".into());
        task.due_date = Some(date(2026, 11, 12));
        app.tasks.push(task);
        app.events
            .push(Event::new("Reunião".into(), date(2026, 11, 12)));
        app.events
            .push(Event::new("Festa".into(), date(2026, 11, 20)));

        let today = date(2026, 11, 11);
        let both = day_markers(&app, date(2026, 11, 12), today);
        assert!(both.has_tasks && both.has_events);

        let event_only = day_markers(&app, date(2026, 11, 20), today);
        assert!(!event_only.has_tasks && event_only.has_events);
    }

    #[test]
    fn recurring_task_due_date_is_ignored_for_overdue() {
        let mut app = App::new();
        let mut task = Task::new("Revisão".into());
        task.recurrence = TaskRecurrence::Daily;
        task.due_date = Some(date(2026, 11, 1));
        app.tasks.push(task);

        let today = date(2026, 11, 11);
        let markers = day_markers(&app, date(2026, 11, 1), today);
        assert!(!markers.has_overdue && !markers.has_tasks);
    }
}
