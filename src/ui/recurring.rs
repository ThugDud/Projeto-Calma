use chrono::Datelike;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::models::task::TaskStatus;
use crate::services::calendar::DIAS_SEMANA;
use crate::ui::theme::Theme;

/// Renderiza a seção de tarefas recorrentes em colunas de segunda a domingo.
pub fn render(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let dates = app.recurring_week_dates();
    let first = dates[0];
    let last = dates[6];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style())
        .title(format!(
            " Tarefas Recorrentes — Semana {:02}/{:02} a {:02}/{:02} ",
            first.day(),
            first.month(),
            last.day(),
            last.month()
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let constraints: Vec<Constraint> = (0..7).map(|_| Constraint::Ratio(1, 7)).collect();
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner);

    for (idx, column) in columns.iter().enumerate() {
        render_day_column(f, *column, theme, app, idx, dates[idx]);
    }
}

fn render_day_column(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    app: &App,
    weekday_idx: usize,
    date: chrono::NaiveDate,
) {
    let is_today = date == app.today;
    let is_selected = weekday_idx == app.recurring_selected_day;

    let border_style = if is_today {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default().fg(theme.due_today_fg)
    } else {
        theme.border_style()
    };

    let title_style = if is_today {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default().fg(theme.due_today_fg)
    } else {
        Style::default().fg(theme.muted_fg)
    };

    let title = if is_today {
        format!(" {} {} • ", DIAS_SEMANA[weekday_idx], date.day())
    } else {
        format!(" {} {} ", DIAS_SEMANA[weekday_idx], date.day())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(if is_today {
            Style::default().bg(theme.header_bg).fg(theme.fg)
        } else {
            theme.style()
        })
        .title(Span::styled(title, title_style));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let tasks = app.recurring_tasks_for_day(date);
    let mut lines: Vec<Line> = Vec::new();

    if tasks.is_empty() {
        lines.push(Line::from(Span::styled("(vazio)", theme.muted_style())));
    } else {
        let width = inner.width as usize;
        for (task_idx, task) in tasks.iter().enumerate() {
            let selected = is_selected && task_idx == app.recurring_selected_idx;
            let status = app.task_status_on(task, date);
            let (icon, icon_color) = match status {
                TaskStatus::Concluida => ("[✓]", theme.completed_fg),
                TaskStatus::EmAndamento => ("[~]", theme.due_today_fg),
                TaskStatus::Pendente => ("[ ]", theme.fg),
            };

            let time = task
                .scheduled_time
                .map(|t| format!("{} ", t.format("%H:%M")))
                .unwrap_or_default();
            let text = fit(&format!("{}{}", time, task.title), width.saturating_sub(4));

            let text_style = if selected {
                Style::default()
                    .bg(theme.selected_bg)
                    .fg(theme.selected_fg)
                    .add_modifier(Modifier::BOLD)
            } else if status == TaskStatus::Concluida {
                Style::default().fg(theme.completed_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            lines.push(Line::from(vec![
                Span::styled(icon, Style::default().fg(icon_color)),
                Span::raw(" "),
                Span::styled(text, text_style),
            ]));
        }
    }

    f.render_widget(Paragraph::new(Text::from(lines)), inner);
}

/// Ajusta o texto ao espaço disponível, usando reticências quando necessário.
fn fit(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let count = text.chars().count();
    if count <= width {
        return text.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }
    let mut fitted: String = text.chars().take(width - 1).collect();
    fitted.push('…');
    fitted
}

#[cfg(test)]
mod tests {
    use super::fit;

    #[test]
    fn fit_keeps_short_text() {
        assert_eq!(fit("Academia", 10), "Academia");
    }

    #[test]
    fn fit_truncates_with_ellipsis() {
        assert_eq!(fit("Academia", 5), "Acad…");
        assert_eq!(fit("Academia", 1), "…");
        assert_eq!(fit("Academia", 0), "");
    }
}
