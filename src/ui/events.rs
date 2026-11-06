use crate::app::App;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

pub fn render(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style())
        .title(" Agenda de Eventos (n/E: Novo | e: Editar | d: Excluir | ↑↓/jk: Navegar) ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.events.is_empty() {
        let empty = Paragraph::new(
            "Nenhum evento cadastrado.\n\nPressione 'n' ou 'E' para criar um evento (ex: 'Reunião amanhã 14:30').",
        )
        .style(theme.muted_style())
        .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty, inner);
        return;
    }

    let header_cells = vec![
        Span::styled(" Data", Style::default().fg(theme.muted_fg).bold()),
        Span::styled("Horário", Style::default().fg(theme.muted_fg).bold()),
        Span::styled("Título", Style::default().fg(theme.muted_fg).bold()),
        Span::styled(
            "Local/Categoria",
            Style::default().fg(theme.muted_fg).bold(),
        ),
    ];
    let header_row = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .events
        .iter()
        .enumerate()
        .map(|(idx, event)| {
            let is_selected = idx == app.selected_event_index;
            let is_past = event.is_past(app.today);
            let is_today = event.date == app.today;

            let date_color = if is_today {
                theme.due_today_fg
            } else if is_past {
                theme.muted_fg
            } else {
                theme.fg
            };

            let row_style = if is_selected {
                Style::default()
                    .bg(theme.selected_bg)
                    .fg(theme.selected_fg)
                    .bold()
            } else if is_past {
                Style::default().fg(theme.muted_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let marker = if is_today { "• " } else { "  " };
            let local = event
                .location
                .clone()
                .or_else(|| event.category.clone())
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Span::styled(
                    format!(" {}{}", marker, event.date.format("%d/%m/%Y")),
                    Style::default().fg(date_color),
                ),
                Span::styled(
                    format!(" {}", event.time_label()),
                    Style::default().fg(if is_today {
                        theme.due_today_fg
                    } else {
                        theme.muted_fg
                    }),
                ),
                Span::styled(format!(" {}", event.title), row_style),
                Span::styled(format!(" {}", local), Style::default().fg(theme.muted_fg)),
            ])
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(13),
        Constraint::Length(11),
        Constraint::Percentage(45),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .column_spacing(1);

    f.render_widget(table, inner);
}
