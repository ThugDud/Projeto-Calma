use crate::app::{App, SettingId, SettingsSection};
use crate::desktop::notifications::{DesktopDiagnostics, ServiceAvailability};
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Renderiza a tela de configurações persistentes.
pub fn render(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style())
        .title(
            " Configurações (↑↓/jk: Navegar | Enter/Espaço: Alternar/Editar | r: Atualizar ambiente | Esc: Voltar) ",
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let footer_height = if app.settings.feedback.is_some() || app.config.load_warning.is_some() {
        2
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(footer_height)])
        .split(inner);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut selected_line = 0usize;
    let mut current_section: Option<SettingsSection> = None;

    for (index, setting) in SettingId::ALL.iter().enumerate() {
        let section = setting.section();
        if current_section != Some(section) {
            if current_section.is_some() {
                lines.push(Line::from(""));
            }
            lines.push(section_line(section, theme));
            current_section = Some(section);
        }

        let is_selected = index == app.settings.selected;
        if is_selected {
            selected_line = lines.len();
        }

        lines.push(setting_line(app, *setting, is_selected, theme));

        // A indisponibilidade da integração com o sistema precisa ficar
        // explícita, não apenas quando a opção estiver selecionada.
        if *setting == SettingId::StartWithSystem || is_selected {
            lines.push(Line::from(Span::styled(
                format!("      {}", setting.hint()),
                Style::default().fg(theme.muted_fg).italic(),
            )));
        }
    }

    // Seção somente leitura com diagnóstico do ambiente gráfico.
    lines.push(Line::from(""));
    lines.push(section_line(SettingsSection::Desktop, theme));
    lines.extend(desktop_diagnostic_lines(&app.desktop, theme));
    lines.push(Line::from(Span::styled(
        "      Pressione r para atualizar a detecção e a disponibilidade do serviço.",
        Style::default().fg(theme.muted_fg).italic(),
    )));

    // Seção reservada para preferências visuais futuras.
    if current_section != Some(SettingsSection::Appearance) {
        lines.push(Line::from(""));
        lines.push(section_line(SettingsSection::Appearance, theme));
    }
    lines.push(Line::from(vec![
        Span::styled("      Tema atual: ", Style::default().fg(theme.muted_fg)),
        Span::styled(
            app.config.interface.theme.clone(),
            Style::default().fg(theme.fg).bold(),
        ),
        Span::styled(
            " — seletor de temas e preferências visuais em breve.",
            Style::default().fg(theme.muted_fg).italic(),
        ),
    ]));

    let visible_height = chunks[0].height.max(1) as usize;
    let start = if selected_line >= visible_height {
        selected_line + 1 - visible_height
    } else {
        0
    };
    let visible_lines: Vec<Line> = lines.into_iter().skip(start).take(visible_height).collect();

    let content = Paragraph::new(Text::from(visible_lines)).wrap(Wrap { trim: false });
    f.render_widget(content, chunks[0]);

    if footer_height > 0 {
        render_footer(f, chunks[1], theme, app);
    }
}

fn section_line(section: SettingsSection, theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(
        format!(" {} ", section.title()),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))
}

fn setting_line(app: &App, setting: SettingId, is_selected: bool, theme: &Theme) -> Line<'static> {
    let prefix = if is_selected { "  ▶ " } else { "    " };
    let row_style = if is_selected {
        theme.selected_style().add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg)
    };

    let value = if is_selected {
        if let Some(buffer) = &app.settings.editing_value {
            format!("[ {}_ ]", buffer)
        } else {
            setting.value(&app.config)
        }
    } else {
        setting.value(&app.config)
    };

    let value_color = match setting.bool_value(&app.config) {
        Some(true) => theme.completed_fg,
        Some(false) => theme.muted_fg,
        None => theme.due_today_fg,
    };

    Line::from(vec![
        Span::styled(prefix.to_string(), row_style),
        Span::styled(format!("{:<42}", setting.label()), row_style),
        Span::styled(
            value,
            Style::default()
                .fg(value_color)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn desktop_diagnostic_lines(diagnostics: &DesktopDiagnostics, theme: &Theme) -> Vec<Line<'static>> {
    let service_color = match diagnostics.availability {
        ServiceAvailability::Available => theme.completed_fg,
        ServiceAvailability::Unavailable => theme.overdue_fg,
        ServiceAvailability::Unknown => theme.due_today_fg,
    };

    let delivery_color = if !diagnostics.delivery_enabled {
        theme.muted_fg
    } else if diagnostics.availability == ServiceAvailability::Available {
        theme.completed_fg
    } else {
        theme.overdue_fg
    };

    vec![
        diagnostic_line(
            "Desktop detectado",
            &diagnostics.desktop_label(),
            theme.fg,
            theme,
        ),
        diagnostic_line("Sessão", diagnostics.session_label(), theme.fg, theme),
        diagnostic_line("Backend", diagnostics.backend_label(), theme.fg, theme),
        diagnostic_line(
            "Serviço de notificações",
            diagnostics.service_label(),
            service_color,
            theme,
        ),
        diagnostic_line(
            "Entrega de notificações",
            diagnostics.delivery_label(),
            delivery_color,
            theme,
        ),
        diagnostic_line(
            "Diagnóstico",
            &diagnostics.diagnostic,
            theme.muted_fg,
            theme,
        ),
    ]
}

fn diagnostic_line(
    label: &str,
    value: &str,
    value_color: ratatui::style::Color,
    theme: &Theme,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("      {:<28}", label),
            Style::default().fg(theme.muted_fg),
        ),
        Span::styled(value.to_string(), Style::default().fg(value_color).bold()),
    ])
}

fn render_footer(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let mut lines = Vec::new();

    if let Some(feedback) = &app.settings.feedback {
        let color = if feedback.starts_with("Configurações salvas")
            || feedback.starts_with("Diagnóstico")
        {
            theme.completed_fg
        } else {
            theme.overdue_fg
        };
        lines.push(Line::from(Span::styled(
            feedback.clone(),
            Style::default().fg(color).bold(),
        )));
    }

    if let Some(warning) = &app.config.load_warning {
        lines.push(Line::from(Span::styled(
            format!("Aviso de configuração: {}", warning),
            Style::default().fg(theme.due_today_fg),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::detection::{DesktopEnvironment, DesktopSession, SessionType};
    use crate::desktop::notifications::{BackendKind, ServiceAvailability};

    fn diagnostics(
        environment: DesktopEnvironment,
        session_type: SessionType,
        backend: BackendKind,
        availability: ServiceAvailability,
        delivery_enabled: bool,
    ) -> DesktopDiagnostics {
        DesktopDiagnostics {
            session: DesktopSession {
                environment,
                session_type,
                raw_desktop: None,
            },
            backend,
            availability,
            delivery_enabled,
            diagnostic: "mensagem de diagnóstico".to_string(),
        }
    }

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn rendered_text(diagnostics: &DesktopDiagnostics) -> Vec<String> {
        desktop_diagnostic_lines(diagnostics, &Theme::default())
            .iter()
            .map(line_text)
            .collect()
    }

    #[test]
    fn renders_detected_desktop_and_session_type() {
        let diagnostics = diagnostics(
            DesktopEnvironment::KdePlasma,
            SessionType::Wayland,
            BackendKind::Kde,
            ServiceAvailability::Available,
            true,
        );

        let text = rendered_text(&diagnostics);
        assert!(
            text.iter()
                .any(|line| line.contains("Desktop detectado") && line.contains("KDE Plasma"))
        );
        assert!(
            text.iter()
                .any(|line| line.contains("Sessão") && line.contains("Wayland"))
        );
        assert!(
            text.iter()
                .any(|line| line.contains("Backend") && line.contains("KDE / Freedesktop D-Bus"))
        );
        assert!(
            text.iter()
                .any(|line| line.contains("Serviço de notificações") && line.contains("Disponível"))
        );
        assert!(
            text.iter()
                .any(|line| line.contains("Entrega de notificações") && line.contains("Ativada"))
        );
    }

    #[test]
    fn renders_headless_session_without_misleading_success() {
        let diagnostics = diagnostics(
            DesktopEnvironment::Headless,
            SessionType::Headless,
            BackendKind::Noop,
            ServiceAvailability::Unavailable,
            true,
        );

        let text = rendered_text(&diagnostics);
        assert!(
            text.iter()
                .any(|line| line.contains("Desktop detectado") && line.contains("Nenhum"))
        );
        assert!(
            text.iter()
                .any(|line| line.contains("Sessão") && line.contains("Headless"))
        );
        assert!(
            text.iter()
                .any(|line| line.contains("Serviço de notificações")
                    && line.contains("Indisponível"))
        );
        assert!(
            text.iter()
                .any(|line| line.contains("sem backend disponível"))
        );
    }

    #[test]
    fn renders_disabled_delivery_explicitly() {
        let diagnostics = diagnostics(
            DesktopEnvironment::Gnome,
            SessionType::X11,
            BackendKind::Freedesktop,
            ServiceAvailability::Available,
            false,
        );

        let text = rendered_text(&diagnostics);
        assert!(
            text.iter()
                .any(|line| line.contains("Entrega de notificações")
                    && line.contains("Desativada (configuração)"))
        );
        assert!(
            text.iter()
                .any(|line| line.contains("mensagem de diagnóstico"))
        );
    }
}
