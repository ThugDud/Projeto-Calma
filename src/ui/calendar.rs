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
            "  (Nenhuma tarefa com vencimento nesta data)",
            theme.muted_style(),
        )));
    } else {
        for (idx, t) in day_tasks.iter().enumerate() {
            let is_selected = idx == app.calendar_selected_task_idx;
            let (icon, color) = match t.status {
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
            side_lines.push(Line::from(vec![
                Span::styled(
                    if is_selected { "  > " } else { "    " },
                    Style::default().fg(theme.accent),
                ),
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(t.title.clone(), title_style),
            ]));
        }
    }

    let side_p = Paragraph::new(Text::from(side_lines));
    f.render_widget(side_p, side_inner);
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

            // Conta tarefas nesta data
            let has_tasks = app
                .tasks
                .iter()
                .any(|t| t.due_date == Some(*date) && t.status != TaskStatus::Concluida);
            let has_overdue = app.tasks.iter().any(|t| {
                t.due_date == Some(*date) && *date < today && t.status != TaskStatus::Concluida
            });

            let marker = if has_overdue {
                "!"
            } else if has_tasks {
                "•"
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
            } else if has_tasks {
                Style::default().fg(Color::Rgb(0xeb, 0xcb, 0x8b))
            } else {
                Style::default().fg(theme.fg)
            };
            Span::styled(text, st)
        }
        None => Span::styled("    ", Style::default().fg(theme.muted_fg)),
    }
}
