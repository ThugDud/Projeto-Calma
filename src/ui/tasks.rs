use crate::app::App;
use crate::models::task::{TaskPriority, TaskStatus};
use crate::services::date_parser::format_due_label;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Row, Table};

pub fn render(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style())
        .title(" Lista de Tarefas (n: Nova | Espaço: Concluir | d: Excluir | ↑↓: Navegar) ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.tasks.is_empty() {
        let empty_msg = ratatui::widgets::Paragraph::new(
            "Nenhuma tarefa cadastrada. Pressione 'n' para cadastrar uma nova tarefa!",
        )
        .style(theme.muted_style())
        .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty_msg, inner);
        return;
    }

    let header_cells = vec![
        Span::styled(" St ", Style::default().fg(theme.muted_fg).bold()),
        Span::styled("Título", Style::default().fg(theme.muted_fg).bold()),
        Span::styled("Prioridade", Style::default().fg(theme.muted_fg).bold()),
        Span::styled("Prazo", Style::default().fg(theme.muted_fg).bold()),
        Span::styled("Situação", Style::default().fg(theme.muted_fg).bold()),
        Span::styled(
            "Categoria/Projeto",
            Style::default().fg(theme.muted_fg).bold(),
        ),
    ];
    let header_row = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let is_selected = idx == app.selected_index;
            let (status_icon, status_color) = match task.status {
                TaskStatus::Concluida => ("[✓]", theme.completed_fg),
                TaskStatus::EmAndamento => ("[~]", Color::Rgb(0xeb, 0xcb, 0x8b)),
                TaskStatus::Pendente => ("[ ]", theme.fg),
            };

            let (prio_label, prio_color) = match task.priority {
                TaskPriority::Urgente => ("URGENTE", Color::Rgb(0xbf, 0x61, 0x6a)),
                TaskPriority::Alta => ("Alta", Color::Rgb(0xd0, 0x87, 0x70)),
                TaskPriority::Normal => ("Normal", theme.fg),
                TaskPriority::Baixa => ("Baixa", theme.muted_fg),
            };

            let is_completed = task.status == TaskStatus::Concluida;
            let (due_text, due_kind) = format_due_label(task.due_date, app.today, is_completed);
            let due_color = match due_kind {
                "overdue" => theme.overdue_fg,
                "due_today" => theme.due_today_fg,
                "due_soon" => Color::Rgb(0xeb, 0xcb, 0x8b),
                "completed" => theme.completed_fg,
                _ => theme.muted_fg,
            };

            let due_date_str = task
                .due_date
                .map(|d| d.format("%d/%m/%Y").to_string())
                .unwrap_or_else(|| "-".to_string());

            let project_name = task
                .category
                .clone()
                .or_else(|| {
                    task.project_id.and_then(|pid| {
                        app.projects
                            .iter()
                            .find(|p| p.id == pid)
                            .map(|p| p.name.clone())
                    })
                })
                .unwrap_or_else(|| "-".to_string());

            let row_style = if is_selected {
                Style::default()
                    .bg(theme.selected_bg)
                    .fg(theme.selected_fg)
                    .bold()
            } else if is_completed {
                Style::default().fg(theme.muted_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            Row::new(vec![
                Span::styled(
                    format!(" {} ", status_icon),
                    Style::default().fg(status_color),
                ),
                Span::styled(format!(" {}", task.title), row_style),
                Span::styled(format!(" {}", prio_label), Style::default().fg(prio_color)),
                Span::styled(format!(" {}", due_date_str), row_style),
                Span::styled(format!(" {}", due_text), Style::default().fg(due_color)),
                Span::styled(
                    format!(" {}", project_name),
                    Style::default().fg(theme.muted_fg),
                ),
            ])
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Percentage(35),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .column_spacing(1);

    f.render_widget(table, inner);
}
