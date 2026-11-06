use crate::app::App;
use crate::models::task::TaskStatus;
use crate::services::calendar::{DIAS_SEMANA_COMPLETO, MESES};
use crate::ui::theme::Theme;
use chrono::Datelike;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Renderiza o cabeçalho da aplicação.
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    theme: &Theme,
    today: chrono::NaiveDate,
    app: &App,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.header_style());

    let weekday_idx = today.weekday().num_days_from_monday() as usize;
    let dia_semana = DIAS_SEMANA_COMPLETO[weekday_idx];

    let mes_idx = (today.month() - 1) as usize;
    let mes_ano = format!("{} de {}", MESES[mes_idx], today.year());

    let total = app.tasks.len();
    let completed = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Concluida)
        .count();
    let overdue = app
        .tasks
        .iter()
        .filter(|t| t.status != TaskStatus::Concluida && t.due_date.is_some_and(|d| d < today))
        .count();
    let due_today = app
        .tasks
        .iter()
        .filter(|t| t.status != TaskStatus::Concluida && t.due_date == Some(today))
        .count();

    let title_span = Span::styled(
        " CALMA ",
        Style::default()
            .bg(Color::Rgb(0x8f, 0xbc, 0xbb))
            .fg(Color::Rgb(0x1e, 0x1e, 0x1e))
            .bold(),
    );

    let view_name = match app.current_view {
        crate::app::View::Calendario => "[1: Calendário]",
        crate::app::View::Tarefas => "[2: Tarefas]",
        crate::app::View::Hierarquia => "[3: Hierarquia]",
        crate::app::View::VisaoGeral => "[4: Visão Geral]",
        crate::app::View::Eventos => "[5: Eventos]",
        crate::app::View::Configuracoes => "[6: Configurações]",
    };

    let date_line = format!(
        "{} | {:02}/{:02}/{} | {}   |   Visualização: {}",
        dia_semana,
        today.day(),
        today.month(),
        today.year(),
        mes_ano,
        view_name
    );

    let stats_spans = vec![
        Span::styled(format!("Total: {}  ", total), Style::default().fg(theme.fg)),
        Span::styled(
            format!("✓ Concluídas: {}  ", completed),
            Style::default().fg(theme.completed_fg),
        ),
        Span::styled(
            format!("⚡ Hoje: {}  ", due_today),
            Style::default().fg(theme.due_today_fg),
        ),
        Span::styled(
            format!("⚠ Atrasadas: {}", overdue),
            Style::default().fg(if overdue > 0 {
                theme.overdue_fg
            } else {
                theme.muted_fg
            }),
        ),
    ];

    let lines = vec![
        Line::from(vec![
            title_span,
            Span::raw(" "),
            Span::styled(date_line, Style::default().fg(theme.fg).bold()),
        ]),
        Line::from(stats_spans),
    ];

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
