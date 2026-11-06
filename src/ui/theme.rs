use ratatui::style::{Color, Style};
use ratatui::text::Span;

/// Paleta de cores do tema escuro padrão do Calma.
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub today_bg: Color,
    pub today_fg: Color,
    pub overdue_fg: Color,
    pub due_today_fg: Color,
    pub due_tomorrow_fg: Color,
    pub completed_fg: Color,
    pub muted_fg: Color,
    pub border: Color,
    pub accent: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            bg: Color::Rgb(0x1e, 0x1e, 0x1e), // fundo escuro
            fg: Color::Rgb(0xe0, 0xe0, 0xe0), // texto principal
            header_bg: Color::Rgb(0x26, 0x26, 0x26),
            header_fg: Color::Rgb(0x8f, 0xbc, 0xbb), // turquesa suave
            selected_bg: Color::Rgb(0x4c, 0x4c, 0x4c),
            selected_fg: Color::Rgb(0x8f, 0xbc, 0xbb),
            today_bg: Color::Rgb(0x5e, 0x81, 0xac), // azul discreto
            today_fg: Color::Rgb(0xff, 0xff, 0xff),
            overdue_fg: Color::Rgb(0xbf, 0x61, 0x6a), // vermelho suave
            due_today_fg: Color::Rgb(0xeb, 0xcb, 0x8b), // amarelo
            due_tomorrow_fg: Color::Rgb(0xeb, 0xcb, 0x8b),
            completed_fg: Color::Rgb(0xa3, 0xbe, 0x8c), // verde
            muted_fg: Color::Rgb(0x65, 0x65, 0x65),
            border: Color::Rgb(0x4c, 0x4c, 0x4c),
            accent: Color::Rgb(0x8f, 0xbc, 0xbb),
        }
    }
}

impl Theme {
    pub fn style(&self) -> Style {
        Style::default().bg(self.bg).fg(self.fg)
    }

    pub fn header_style(&self) -> Style {
        Style::default().bg(self.header_bg).fg(self.header_fg)
    }

    pub fn selected_style(&self) -> Style {
        Style::default().bg(self.selected_bg).fg(self.selected_fg)
    }

    pub fn today_style(&self) -> Style {
        Style::default().bg(self.today_bg).fg(self.today_fg)
    }

    pub fn overdue_style(&self) -> Style {
        Style::default().fg(self.overdue_fg)
    }

    pub fn due_today_style(&self) -> Style {
        Style::default().fg(self.due_today_fg)
    }

    pub fn completed_style(&self) -> Style {
        Style::default().fg(self.completed_fg)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted_fg)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }
}

/// Cria um Span de texto com um estilo aplicado.
pub fn span(text: &str, style: Style) -> Span<'static> {
    Span::styled(text.to_string(), style)
}
