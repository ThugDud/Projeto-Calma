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
            "CADASTRO RÁPIDO: 'titulo [prazo|diariamente|seg,qua,sex] [hora]' = tarefa | 'event:titulo data [hora]' = evento | [Enter] Salvar | [Esc] Cancelar"
        }
        InputMode::QuickAddEvent => {
            "NOVO EVENTO: 'titulo data [hora]' (ex: 'Reunião amanhã 14:30') | [Enter] Salvar | [Esc] Cancelar"
        }
        InputMode::ConfirmDelete { .. } => {
            "CONFIRMAR EXCLUSÃO: [s / Enter] Confirmar Exclusão | [n / Esc] Cancelar"
        }
        InputMode::EditTask(_) => {
            "EDITAR: [Tab] Campo | Repetição [0-2] | Dias [1-7] | Prioridade [1-4] | [p] Alternar Projeto | [Enter] Salvar | [Esc] Cancelar"
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
        InputMode::ExportData { .. } => {
            "EXPORTAR DADOS: Edite o caminho JSON | [Enter] Exportar | [Esc] Cancelar"
        }
        InputMode::ImportData { .. } => {
            "IMPORTAR DADOS: Informe o caminho do arquivo JSON | [Enter] Continuar | [Esc] Cancelar"
        }
        InputMode::ImportChoice { .. } => {
            "MODO DE IMPORTAÇÃO: [↑↓/Espaço] Alternar | [a] Adicionar | [o] Sobrescrever | [Enter] Confirmar | [Esc] Cancelar"
        }
        InputMode::Normal => match app.current_view {
            View::Calendario => {
                "[Tab] Menu de seções | [n] Nova | [E] Evento | [j/k] Selecionar | [t] Hoje | [x] Exportar | [i] Importar | [?] Ajuda | [q] Sair"
            }
            View::Tarefas => {
                "[Tab] Menu de seções | [n] Nova | [E] Evento | [e] Editar | [Espaço] Concluir | [d] Excluir | [x] Exportar | [i] Importar | [?] Ajuda | [q] Sair"
            }
            View::Hierarquia => {
                "[Tab] Menu de seções | [Enter] Exp/Rec | [s] Subtarefa | [p] Projeto | [e] Editar | [Espaço] Concluir | [d] Excluir | [x] Exportar | [i] Importar | [?] Ajuda | [q] Sair"
            }
            View::VisaoGeral => {
                "[Tab] Menu de seções | [↑↓/jk] Selecionar | [E] Evento | [e] Editar | [Espaço] Concluir | [d] Excluir | [x] Exportar | [i] Importar | [?] Ajuda | [q] Sair"
            }
            View::Eventos => {
                "[Tab] Menu de seções | [n/E] Novo evento | [e] Editar | [d] Excluir | [↑↓/jk] Navegar | [x] Exportar | [i] Importar | [?] Ajuda | [q] Sair"
            }
            View::Recorrentes => {
                "[Tab] Menu de seções | [←/→] Dia | [↑↓/jk] Tarefa | [Espaço] Concluir | [e] Editar | [d] Excluir | [t] Hoje | [?] Ajuda | [q] Sair"
            }
            View::Configuracoes => {
                "[Tab] Menu de seções | [↑↓/jk] Navegar | [Enter/Espaço] Alternar ou editar | [r] Atualizar ambiente | [x] Exportar | [i] Importar | [Esc] Voltar | [?] Ajuda | [q] Sair"
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
