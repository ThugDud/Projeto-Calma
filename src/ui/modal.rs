use crate::app::{App, DeleteTarget, EventEditState, TaskEditState};
use crate::models::task::TaskPriority;
use crate::services::date_parser::{
    parse_pt_date, parse_quick_event, parse_quick_task, parse_time,
};
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn render_quick_add(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    // O prefixo "event:" continua funcionando no cadastro rápido global.
    if app.input_buffer.trim().to_lowercase().starts_with("event:") {
        render_quick_add_event(f, area, theme, app);
        return;
    }

    let popup_area = centered_rect(65, 30, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)))
        .style(theme.style())
        .title(" [ Nova Tarefa Rápida ] ");

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let parsed = parse_quick_task(&app.input_buffer, app.today);

    let parsed_preview = if let Some(d) = parsed.due_date {
        format!(
            "Data interpretada: {} ({})",
            d.format("%d/%m/%Y"),
            d.format("%A")
        )
    } else {
        "Sem data especificada (pressione Enter para salvar sem data)".to_string()
    };

    let lines = vec![
        Line::from(Span::styled(
            "Digite a tarefa (ex: 'Estudar C amanhã', 'Pagar conta dia 5'):",
            theme.muted_style(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " > ",
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
            ),
            Span::styled(
                format!("{}_", app.input_buffer),
                Style::default().fg(theme.fg).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Interpretação: ", Style::default().fg(theme.muted_fg)),
            Span::styled(
                parsed_preview,
                Style::default().fg(Color::Rgb(0xeb, 0xcb, 0x8b)),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Confirmar e Salvar   |   [Esc] Cancelar",
            Style::default().fg(theme.muted_fg),
        )),
    ];

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);
    f.render_widget(p, inner);
}

pub fn render_quick_add_event(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let popup_area = centered_rect(65, 32, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)))
        .style(theme.style())
        .title(" [ Novo Evento ] ");

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let raw = app.input_buffer.trim();
    let input = if raw.to_lowercase().starts_with("event:") {
        raw[6..].trim()
    } else {
        raw
    };
    let parsed = parse_quick_event(input, app.today);

    let preview = if input.is_empty() {
        "Digite o título, a data (opcional) e o horário (opcional).".to_string()
    } else {
        let time = match (parsed.start_time, parsed.end_time) {
            (Some(start), Some(end)) => {
                format!("{}-{}", start.format("%H:%M"), end.format("%H:%M"))
            }
            (Some(start), None) => start.format("%H:%M").to_string(),
            _ => "sem horário".to_string(),
        };
        format!(
            "Título: {} | Data: {} ({}) | Horário: {}",
            if parsed.title.is_empty() {
                "(sem título)"
            } else {
                &parsed.title
            },
            parsed.date.format("%d/%m/%Y"),
            parsed.date.format("%A"),
            time
        )
    };

    let lines = vec![
        Line::from(Span::styled(
            "Digite o evento (ex: 'Reunião amanhã 14:30-16:00', 'Consulta dia 5 9h'):",
            theme.muted_style(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " > ",
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
            ),
            Span::styled(
                format!("{}_", app.input_buffer),
                Style::default().fg(theme.fg).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Interpretação: ", Style::default().fg(theme.muted_fg)),
            Span::styled(preview, Style::default().fg(Color::Rgb(0xeb, 0xcb, 0x8b))),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Confirmar e Salvar   |   [Esc] Cancelar",
            Style::default().fg(theme.muted_fg),
        )),
    ];

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);
    f.render_widget(p, inner);
}

/// Renderiza o diálogo de confirmação antes de excluir (Prioridade 2).
pub fn render_confirm_delete(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    target: DeleteTarget,
    title: &str,
) {
    let popup_area = centered_rect(60, 24, area);
    f.render_widget(Clear, popup_area);

    let border_color = Color::Rgb(0xbf, 0x61, 0x6a); // vermelho suave
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color).bold())
        .style(theme.style())
        .title(" [ Confirmar Exclusão ] ");

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let (item_type, warning) = match target {
        DeleteTarget::Project => (
            "o projeto",
            "Aviso: tarefas e subprojetos vinculados serão desvinculados.",
        ),
        DeleteTarget::Event => ("o evento", "Esta ação removerá o evento da sua agenda."),
        DeleteTarget::Task => (
            "a tarefa",
            "Esta ação removerá o registro do banco SQLite local.",
        ),
    };

    let lines = vec![
        Line::from(Span::styled(
            format!("Tem certeza que deseja excluir {} abaixo?", item_type),
            Style::default().fg(theme.fg),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  \" ", Style::default().fg(border_color).bold()),
            Span::styled(
                title.to_string(),
                Style::default().fg(Color::Rgb(0xff, 0xff, 0xff)).bold(),
            ),
            Span::styled(" \"", Style::default().fg(border_color).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            warning,
            Style::default().fg(theme.muted_fg).italic(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[s / Enter] ", Style::default().fg(border_color).bold()),
            Span::styled("Confirmar Exclusão   |   ", Style::default().fg(theme.fg)),
            Span::styled(
                "[n / Esc] ",
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
            ),
            Span::styled("Cancelar", Style::default().fg(theme.fg)),
        ]),
    ];

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Center);
    f.render_widget(p, inner);
}

/// Renderiza formulário funcional para edição de tarefa (Prioridade 3).
pub fn render_edit_task(f: &mut Frame, area: Rect, theme: &Theme, app: &App, edit: &TaskEditState) {
    let popup_area = centered_rect(70, 50, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold())
        .style(theme.style())
        .title(" [ Editar Tarefa ] ");

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Utilize [Tab] para alternar entre campos  |  [Enter] Salvar  |  [Esc] Cancelar",
        theme.muted_style(),
    )));
    lines.push(Line::from(""));

    // Campo 0: Título
    let title_prefix = if edit.active_field == 0 {
        " ▶ Título: "
    } else {
        "   Título: "
    };
    let title_style = if edit.active_field == 0 {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .bold()
    } else {
        Style::default().fg(theme.fg)
    };
    lines.push(Line::from(vec![
        Span::styled(
            title_prefix,
            Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
        ),
        Span::styled(format!("{}_", edit.title), title_style),
    ]));
    lines.push(Line::from(""));

    // Campo 1: Prazo
    let date_prefix = if edit.active_field == 1 {
        " ▶ Prazo:  "
    } else {
        "   Prazo:  "
    };
    let date_style = if edit.active_field == 1 {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .bold()
    } else {
        Style::default().fg(theme.fg)
    };
    let parsed_due = parse_pt_date(&edit.due_date_str, app.today);
    let preview_str = if let Some(d) = parsed_due {
        format!(" -> Reconhecido: {}", d.format("%d/%m/%Y"))
    } else if edit.due_date_str.trim().is_empty() {
        " -> (Sem prazo)".to_string()
    } else {
        " -> (Formato não reconhecido)".to_string()
    };
    lines.push(Line::from(vec![
        Span::styled(
            date_prefix,
            Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
        ),
        Span::styled(format!("{}_", edit.due_date_str), date_style),
        Span::styled(
            preview_str,
            Style::default().fg(Color::Rgb(0xeb, 0xcb, 0x8b)),
        ),
    ]));
    lines.push(Line::from(""));

    // Campo 2: Prioridade
    let prio_prefix = if edit.active_field == 2 {
        " ▶ Prioridade: "
    } else {
        "   Prioridade: "
    };
    let prio_options = [
        TaskPriority::Baixa,
        TaskPriority::Normal,
        TaskPriority::Alta,
        TaskPriority::Urgente,
    ];
    let mut prio_spans = vec![Span::styled(
        prio_prefix,
        Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
    )];
    for p in prio_options {
        let is_current = p == edit.priority;
        let p_name = match p {
            TaskPriority::Baixa => "Baixa (1)",
            TaskPriority::Normal => "Normal (2)",
            TaskPriority::Alta => "Alta (3)",
            TaskPriority::Urgente => "Urgente (4)",
        };
        let p_style = if is_current {
            Style::default()
                .bg(theme.selected_bg)
                .fg(Color::Rgb(0xeb, 0xcb, 0x8b))
                .bold()
        } else {
            Style::default().fg(theme.muted_fg)
        };
        prio_spans.push(Span::styled(format!(" [{}] ", p_name), p_style));
    }
    lines.push(Line::from(prio_spans));
    lines.push(Line::from(""));

    // Campo 3: Projeto
    let proj_prefix = if edit.active_field == 3 {
        " ▶ Projeto: "
    } else {
        "   Projeto: "
    };
    let current_proj_name = edit
        .project_id
        .and_then(|pid| app.projects.iter().find(|p| p.id == pid))
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Nenhum (Geral)".to_string());
    lines.push(Line::from(vec![
        Span::styled(
            proj_prefix,
            Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
        ),
        Span::styled(
            format!(
                "[ {} ] (Pressione 'p' para alternar projeto)",
                current_proj_name
            ),
            if edit.active_field == 3 {
                Style::default()
                    .bg(theme.selected_bg)
                    .fg(theme.selected_fg)
                    .bold()
            } else {
                Style::default().fg(theme.fg)
            },
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Confirmar Alterações   |   [Esc] Cancelar",
        Style::default().fg(theme.muted_fg).bold(),
    )));

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);
    f.render_widget(p, inner);
}

/// Renderiza formulário funcional para edição de evento.
pub fn render_edit_event(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    app: &App,
    edit: &EventEditState,
) {
    let popup_area = centered_rect(70, 50, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold())
        .style(theme.style())
        .title(" [ Editar Evento ] ");

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Utilize [Tab] para alternar entre campos  |  [Enter] Salvar  |  [Esc] Cancelar",
        theme.muted_style(),
    )));
    lines.push(Line::from(""));

    // Campo 0: Título
    let title_prefix = if edit.active_field == 0 {
        " ▶ Título: "
    } else {
        "   Título: "
    };
    let title_style = if edit.active_field == 0 {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .bold()
    } else {
        Style::default().fg(theme.fg)
    };
    lines.push(Line::from(vec![
        Span::styled(
            title_prefix,
            Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
        ),
        Span::styled(format!("{}_", edit.title), title_style),
    ]));
    lines.push(Line::from(""));

    // Campo 1: Data
    let date_prefix = if edit.active_field == 1 {
        " ▶ Data:   "
    } else {
        "   Data:   "
    };
    let date_style = if edit.active_field == 1 {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .bold()
    } else {
        Style::default().fg(theme.fg)
    };
    let date_preview = if let Some(d) = parse_pt_date(&edit.date_str, app.today) {
        format!(" -> Reconhecido: {}", d.format("%d/%m/%Y"))
    } else {
        " -> (Data atual mantida)".to_string()
    };
    lines.push(Line::from(vec![
        Span::styled(
            date_prefix,
            Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
        ),
        Span::styled(format!("{}_", edit.date_str), date_style),
        Span::styled(
            date_preview,
            Style::default().fg(Color::Rgb(0xeb, 0xcb, 0x8b)),
        ),
    ]));
    lines.push(Line::from(""));

    // Campo 2: Horário
    let time_prefix = if edit.active_field == 2 {
        " ▶ Horário: "
    } else {
        "   Horário: "
    };
    let time_style = if edit.active_field == 2 {
        Style::default()
            .bg(theme.selected_bg)
            .fg(theme.selected_fg)
            .bold()
    } else {
        Style::default().fg(theme.fg)
    };
    let time_preview = if edit.time_str.trim().is_empty() {
        " -> (Sem horário)".to_string()
    } else if let Some(t) = parse_time(&edit.time_str) {
        format!(" -> Reconhecido: {}", t.format("%H:%M"))
    } else {
        " -> (Horário atual mantido)".to_string()
    };
    lines.push(Line::from(vec![
        Span::styled(
            time_prefix,
            Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
        ),
        Span::styled(format!("{}_", edit.time_str), time_style),
        Span::styled(
            time_preview,
            Style::default().fg(Color::Rgb(0xeb, 0xcb, 0x8b)),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Confirmar Alterações   |   [Esc] Cancelar",
        Style::default().fg(theme.muted_fg).bold(),
    )));

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);
    f.render_widget(p, inner);
}

/// Renderiza modal de criação de novo projeto ou subprojeto (Prioridade 4).
pub fn render_new_project(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    app: &App,
    parent_id: Option<i64>,
    buffer: &str,
) {
    let popup_area = centered_rect(60, 30, area);
    f.render_widget(Clear, popup_area);

    let title_bar = if let Some(pid) = parent_id {
        let parent_name = app
            .projects
            .iter()
            .find(|p| p.id == pid)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Projeto".to_string());
        format!(" [ Novo Subprojeto de: {} ] ", parent_name)
    } else {
        " [ Novo Projeto Raiz ] ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold())
        .style(theme.style())
        .title(title_bar);

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let lines = vec![
        Line::from(Span::styled(
            "Digite o nome do projeto:",
            theme.muted_style(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " > ",
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
            ),
            Span::styled(format!("{}_", buffer), Style::default().fg(theme.fg).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Criar Projeto   |   [Esc] Cancelar",
            Style::default().fg(theme.muted_fg),
        )),
    ];

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);
    f.render_widget(p, inner);
}

/// Renderiza modal de edição de projeto existente.
pub fn render_edit_project(f: &mut Frame, area: Rect, theme: &Theme, buffer: &str) {
    let popup_area = centered_rect(60, 30, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold())
        .style(theme.style())
        .title(" [ Renomear Projeto ] ");

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let lines = vec![
        Line::from(Span::styled("Novo nome do projeto:", theme.muted_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " > ",
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
            ),
            Span::styled(format!("{}_", buffer), Style::default().fg(theme.fg).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Salvar   |   [Esc] Cancelar",
            Style::default().fg(theme.muted_fg),
        )),
    ];

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);
    f.render_widget(p, inner);
}

/// Renderiza modal de criação de nova subtarefa (Prioridade 4).
pub fn render_new_subtask(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    app: &App,
    parent_title: &str,
    buffer: &str,
) {
    let popup_area = centered_rect(65, 30, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold())
        .style(theme.style())
        .title(format!(" [ Nova Subtarefa de: {} ] ", parent_title));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let parsed = parse_quick_task(buffer, app.today);
    let parsed_preview = if let Some(d) = parsed.due_date {
        format!("Data: {} ({})", d.format("%d/%m/%Y"), d.format("%A"))
    } else {
        "Sem data especificada (herdará da tarefa pai ou ficará sem prazo)".to_string()
    };

    let lines = vec![
        Line::from(Span::styled(
            "Digite a subtarefa (ex: 'Questão 1', 'Revisar amanhã'):",
            theme.muted_style(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " > ",
                Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)).bold(),
            ),
            Span::styled(format!("{}_", buffer), Style::default().fg(theme.fg).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Interpretação: ", Style::default().fg(theme.muted_fg)),
            Span::styled(
                parsed_preview,
                Style::default().fg(Color::Rgb(0xeb, 0xcb, 0x8b)),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Adicionar Subtarefa   |   [Esc] Cancelar",
            Style::default().fg(theme.muted_fg),
        )),
    ];

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);
    f.render_widget(p, inner);
}

pub fn render_help_modal(f: &mut Frame, area: Rect, theme: &Theme) {
    let popup_area = centered_rect(72, 75, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x8f, 0xbc, 0xbb)))
        .style(theme.style())
        .title(" [ Calma - Ajuda e Atalhos ] ");

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let help_lines = vec![
        Line::from(Span::styled(
            "Navegação entre Visualizações:",
            Style::default().bold().fg(Color::Rgb(0x8f, 0xbc, 0xbb)),
        )),
        Line::from("  1 ou m : Calendário Mensal"),
        Line::from("  2 ou l : Lista de Tarefas"),
        Line::from("  3 ou h : Árvore de Projetos e Hierarquia"),
        Line::from("  4 ou v : Visão Geral & Prazos (Dashboard)"),
        Line::from("  5      : Agenda de Eventos"),
        Line::from(""),
        Line::from(Span::styled(
            "Gerenciamento de Tarefas e Projetos:",
            Style::default().bold().fg(Color::Rgb(0x8f, 0xbc, 0xbb)),
        )),
        Line::from("  n      : Nova Tarefa rápida (ou novo Evento na tela de Eventos)"),
        Line::from("  E      : Novo Evento rápido (ex: 'Reunião amanhã 14:30')"),
        Line::from("  s      : Nova Subtarefa (na visualização hierárquica)"),
        Line::from("  p / P  : Novo Projeto ou Subprojeto"),
        Line::from("  e      : Editar tarefa, projeto ou evento selecionado"),
        Line::from("  d      : Excluir item (com confirmação segura antes de apagar)"),
        Line::from("  Espaço : Concluir ou reabrir tarefa selecionada"),
        Line::from("  Enter  : Expandir / Recolher nível na árvore hierárquica"),
        Line::from("  j / k  : Navegar na seleção (calendário, dashboard e eventos)"),
        Line::from(""),
        Line::from(Span::styled(
            "Controle do Calendário:",
            Style::default().bold().fg(Color::Rgb(0x8f, 0xbc, 0xbb)),
        )),
        Line::from("  Setas  : Mover dia (← →) ou semana (↑ ↓)"),
        Line::from("  PgUp   : Mês anterior"),
        Line::from("  PgDn   : Próximo mês"),
        Line::from("  t      : Voltar para Hoje"),
        Line::from(""),
        Line::from(Span::styled(
            "Geral:",
            Style::default().bold().fg(Color::Rgb(0x8f, 0xbc, 0xbb)),
        )),
        Line::from("  ?      : Abrir/Fechar esta tela de ajuda"),
        Line::from("  Esc    : Voltar / Cancelar diálogo"),
        Line::from("  q      : Sair do Calma"),
        Line::from(""),
        Line::from(Span::styled(
            "Pressione Esc ou ? para fechar esta janela.",
            Style::default()
                .fg(theme.muted_fg)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let p = Paragraph::new(Text::from(help_lines));
    f.render_widget(p, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
