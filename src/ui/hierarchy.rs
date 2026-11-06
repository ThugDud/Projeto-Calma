use crate::app::App;
use crate::models::task::{TaskPriority, TaskStatus};
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style())
        .title(" Hierarquia (Enter: Exp/Rec | Espaço: Concluir | s: Subtarefa | p: Subprojeto | e: Editar | d: Excluir) ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let items = app.build_hierarchy_items();

    if items.is_empty() {
        let empty_msg = Paragraph::new(
            "Nenhum projeto ou tarefa cadastrada.\n\nPressione 'n' para criar uma tarefa ou 'p' para criar um projeto!",
        )
        .style(theme.muted_style())
        .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty_msg, inner);
        return;
    }

    let mut lines: Vec<Line<'static>> = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        let is_selected = idx == app.hierarchy_selected_idx;
        let indent = "   ".repeat(item.depth);

        let cursor = if is_selected { " > " } else { "   " };

        let mut spans = Vec::new();
        spans.push(Span::styled(
            cursor.to_string(),
            if is_selected {
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold()
            } else {
                Style::default().fg(theme.muted_fg)
            },
        ));
        spans.push(Span::styled(indent, Style::default().fg(theme.muted_fg)));

        if item.is_project {
            // Ícone do projeto e indicador de expansão
            let exp_icon = if item.has_children {
                if item.is_expanded { "▼ " } else { "▶ " }
            } else {
                "• "
            };
            let folder_icon = if item.is_expanded { "📂 " } else { "📁 " };

            spans.push(Span::styled(
                exp_icon.to_string(),
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)),
            ));
            spans.push(Span::styled(folder_icon.to_string(), Style::default()));

            let name_style = if is_selected {
                Style::default()
                    .bg(theme.selected_bg)
                    .fg(theme.selected_fg)
                    .bold()
            } else {
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold()
            };
            spans.push(Span::styled(format!("{} ", item.title), name_style));

            if let Some((completed, total)) = item.progress {
                let pct = (completed * 100).checked_div(total).unwrap_or(0);
                let bar = format_progress_bar(pct, 8);
                spans.push(Span::styled(
                    format!("[{}] {}% ({}/{})", bar, pct, completed, total),
                    Style::default().fg(theme.muted_fg),
                ));
            }
        } else {
            // É uma tarefa
            let exp_icon = if item.has_children {
                if item.is_expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };
            spans.push(Span::styled(
                exp_icon.to_string(),
                Style::default().fg(theme.muted_fg),
            ));

            let (status_icon, status_color) = match item.status {
                Some(TaskStatus::Concluida) => ("[✓]", theme.completed_fg),
                Some(TaskStatus::EmAndamento) => ("[~]", Color::Rgb(0xeb, 0xcb, 0x8b)),
                _ => ("[ ]", theme.fg),
            };
            spans.push(Span::styled(
                format!("{} ", status_icon),
                Style::default().fg(status_color),
            ));

            let is_completed = item.status == Some(TaskStatus::Concluida);
            let title_style = if is_selected {
                Style::default()
                    .bg(theme.selected_bg)
                    .fg(theme.selected_fg)
                    .bold()
            } else if is_completed {
                Style::default().fg(theme.muted_fg)
            } else {
                Style::default().fg(theme.fg)
            };
            spans.push(Span::styled(format!("{} ", item.title), title_style));

            // Indicador de prioridade
            if let Some(prio) = item.priority {
                let (p_text, p_col) = match prio {
                    TaskPriority::Urgente => ("[URGENTE]", Color::Rgb(0xbf, 0x61, 0x6a)),
                    TaskPriority::Alta => ("[Alta]", Color::Rgb(0xd0, 0x87, 0x70)),
                    TaskPriority::Baixa => ("[Baixa]", theme.muted_fg),
                    TaskPriority::Normal => ("", theme.muted_fg),
                };
                if !p_text.is_empty() {
                    spans.push(Span::styled(
                        format!("{} ", p_text),
                        Style::default().fg(p_col),
                    ));
                }
            }

            // Vencimento
            if let Some(due) = item.due_date {
                let is_overdue = due < app.today && !is_completed;
                let due_col = if is_overdue {
                    theme.overdue_fg
                } else if due == app.today {
                    theme.due_today_fg
                } else {
                    theme.muted_fg
                };
                spans.push(Span::styled(
                    format!("(Vence {}) ", due.format("%d/%m")),
                    Style::default().fg(due_col),
                ));
            }

            // Progresso de subtarefas se houver
            if let Some((completed, total)) = item.progress {
                spans.push(Span::styled(
                    format!("[{}/{}]", completed, total),
                    Style::default().fg(theme.muted_fg),
                ));
            }
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(theme.style());
    f.render_widget(paragraph, inner);
}

fn format_progress_bar(pct: usize, width: usize) -> String {
    let filled = (pct * width) / 100;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
