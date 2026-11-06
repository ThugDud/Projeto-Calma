use crate::app::{App, InputMode, View};
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(theme.style());

    let shortcuts_text = match &app.input_mode {
        InputMode::QuickAdd => {
            "CADASTRO RÁPIDO: 'titulo prazo' = tarefa | 'event:titulo data [hora]' = evento | [Enter] Salvar | [Esc] Cancelar"
        }
        InputMode::QuickAddEvent => {
            "NOVO EVENTO: 'titulo data [hora]' (ex: 'Reunião amanhã 14:30') | [Enter] Salvar | [Esc] Cancelar"
        }
        InputMode::ConfirmDelete { .. } => {
            "CONFIRMAR EXCLUSÃO: [s / Enter] Confirmar Exclusão | [n / Esc] Cancelar"
        }
        InputMode::EditTask(_) => {
            "EDITAR: [Tab] Próximo Campo | [1-4] Prioridade | [p] Alternar Projeto | [Enter] Salvar | [Esc] Cancelar"
        }
        InputMode::EditEvent(_) => {
            "EDITAR EVENTO: [Tab] Próximo Campo | [Enter] Salvar | [Esc] Cancelar"
        }
        InputMode::NewProject { .. } => {
            "NOVO PROJETO: Digite o nome do projeto | [Enter] Criar | [Esc] Cancelar"
        }
        InputMode::EditProject { .. } => {
            "RENOMEAR: Digite o novo nome do projeto | [Enter] Salvar | [Esc] Cancelar"
        }
        InputMode::NewSubtask { .. } => {
            "NOVA SUBTAREFA: Digite título e data da subtarefa | [Enter] Adicionar | [Esc] Cancelar"
        }
        InputMode::Normal => match app.current_view {
            View::Calendario => {
                "[1] Mês [2] Tarefas [3] Hierarquia [4] Visão Geral [5] Eventos [6] Configurações | [n] Nova | [E] Evento | [j/k] Selecionar | [t] Hoje | [PgUp/PgDn] Mês | [?] Ajuda | [q] Sair"
            }
            View::Tarefas => {
                "[1] Mês [2] Tarefas [3] Hierarquia [4] Visão Geral [5] Eventos [6] Configurações | [n] Nova | [E] Evento | [e] Editar | [Espaço] Concluir | [d] Excluir | [?] Ajuda | [q] Sair"
            }
            View::Hierarquia => {
                "[Enter] Exp/Rec | [s] Subtarefa | [p] Projeto | [e] Editar | [Espaço] Concluir | [d] Excluir | [E] Evento | [6] Configurações | [?] Ajuda | [q] Sair"
            }
            View::VisaoGeral => {
                "[↑↓/jk] Selecionar | [E] Evento | [e] Editar | [Espaço] Concluir | [d] Excluir | [5] Eventos [6] Configurações | [?] Ajuda | [q] Sair"
            }
            View::Eventos => {
                "[n/E] Novo evento | [e] Editar | [d] Excluir | [↑↓/jk] Navegar | [1-4/6] Outras telas | [?] Ajuda | [q] Sair"
            }
            View::Configuracoes => {
                "[↑↓/jk] Navegar | [Enter/Espaço] Alternar ou editar | [1-5] Outras telas | [Esc] Voltar | [?] Ajuda | [q] Sair"
            }
        },
    };

    let p = Paragraph::new(Line::from(vec![
        Span::styled(
            "Atalhos: ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(shortcuts_text, Style::default().fg(theme.fg)),
    ]))
    .block(block);

    f.render_widget(p, area);
}
