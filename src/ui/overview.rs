use crate::app::App;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style())
        .title(" Visão Geral & Dashboard de Prazos (↑↓/jk: Navegar) ");

    let inner = main_block.inner(area);
    f.render_widget(main_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let lists = app.overview_lists();
    let selected = app.overview_selected_idx;

    // 1. Tarefas Atrasadas
    let overdue_items: Vec<String> = lists
        .overdue
        .iter()
        .map(|t| {
            let days = app
                .today
                .signed_duration_since(t.due_date.unwrap())
                .num_days();
            format!("• {} (atrasada {}d)", t.title, days)
        })
        .collect();

    render_card(
        f,
        top_chunks[0],
        &format!("⚠ Atrasadas ({})", lists.overdue.len()),
        theme.overdue_fg,
        &overdue_items,
        theme,
        "Nenhuma tarefa atrasada! Parabéns.",
        selected,
        0,
    );

    // 2. Tarefas para Hoje
    let today_items: Vec<String> = lists
        .today
        .iter()
        .map(|t| format!("• {}", t.title))
        .collect();

    render_card(
        f,
        top_chunks[1],
        &format!("⚡ Para Hoje ({})", lists.today.len()),
        theme.due_today_fg,
        &today_items,
        theme,
        "Nenhuma tarefa com vencimento hoje.",
        selected,
        lists.overdue.len(),
    );

    // 3. Próximos 7 Dias
    let upcoming_items: Vec<String> = lists
        .upcoming
        .iter()
        .map(|t| {
            let due = t.due_date.unwrap();
            format!(
                "• {} (em {}d - {})",
                t.title,
                due.signed_duration_since(app.today).num_days(),
                due.format("%d/%m")
            )
        })
        .collect();

    let upcoming_offset = lists.overdue.len() + lists.today.len();
    render_card(
        f,
        bottom_chunks[0],
        &format!("📅 Próximos 7 Dias ({})", lists.upcoming.len()),
        Color::Rgb(0x8f, 0xbc, 0xbb),
        &upcoming_items,
        theme,
        "Sem tarefas para os próximos 7 dias.",
        selected,
        upcoming_offset,
    );

    // 4. Sem Prazo
    let nodate_items: Vec<String> = lists
        .no_date
        .iter()
        .map(|t| format!("• {}", t.title))
        .collect();

    let nodate_offset = upcoming_offset + lists.upcoming.len();
    render_card(
        f,
        bottom_chunks[1],
        &format!("📋 Sem Prazo ({})", lists.no_date.len()),
        theme.muted_fg,
        &nodate_items,
        theme,
        "Todas as tarefas estão agendadas.",
        selected,
        nodate_offset,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_card(
    f: &mut Frame,
    area: Rect,
    title: &str,
    title_color: Color,
    items: &[String],
    theme: &Theme,
    empty_label: &str,
    selected: usize,
    offset: usize,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(title_color).bold(),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if items.is_empty() {
        let p = Paragraph::new(empty_label).style(theme.muted_style());
        f.render_widget(p, inner);
    } else {
        let visible_height = inner.height as usize;
        let selected_in_card = if selected >= offset && selected < offset + items.len() {
            Some(selected - offset)
        } else {
            None
        };
        let start = match selected_in_card {
            Some(si) if visible_height > 0 && si >= visible_height => si + 1 - visible_height,
            _ => 0,
        };

        let lines: Vec<Line> = items
            .iter()
            .enumerate()
            .skip(start)
            .take(visible_height)
            .map(|(idx, item)| {
                let is_selected = offset + idx == selected;
                if is_selected {
                    Line::from(Span::styled(
                        format!("> {}", item.trim_start_matches("• ")),
                        theme.selected_style().add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(item.clone(), Style::default().fg(theme.fg)))
                }
            })
            .collect();
        let p = Paragraph::new(Text::from(lines));
        f.render_widget(p, inner);
    }
}
