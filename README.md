# Calma: Gerenciador Pessoal de Tempo e Tarefas em TUI

O **Calma** é um gerenciador pessoal de tarefas, projetos, compromissos e datas, executado diretamente no terminal Linux por meio de uma interface TUI (*Text User Interface*) moderna, sóbria e focada em produtividade sem atritos ou burocracia.

Desenvolvido em **Rust** com **Ratatui**, **Crossterm** e **SQLite**.

---

## 🎯 Objetivo

Ajudar você a organizar a vida acadêmica, pessoal e financeira sem exigir planejamento burocrático. O objetivo é que registrar uma tarefa seja mais rápido do que deixá-la para depois.

- Tarefas podem existir com ou sem data definida.
- Prazos e compromissos importantes permanecem sempre visíveis.
- Informações atrasadas ganham destaque imediato e nunca desaparecem silenciosamente.
- Controle 100% via teclado no terminal.
- Dados locais salvos em SQLite respeitando convenções Linux (`XDG_DATA_HOME`).

---

## 🚀 Recursos Implementados

1. **Cabeçalho Dinâmico em Português Brasileiro**:
   - Dia da semana em extenso (ex: *Terça-feira*), data completa e mês/ano obtidos do sistema operacional em tempo real.
   - Indicador visual em tempo real do estado geral das tarefas: total, concluídas, tarefas para hoje e atrasadas.
   - Indicador da aba/visualização atual.

2. **Calendário Mensal Visual**:
   - 7 dias da semana (começando na segunda-feira).
   - Navegação por dias e semanas via setas do teclado (`←` `→` `↑` `↓`).
   - Navegação entre meses com `PgUp` / `PgDn`.
   - Destaque para o dia atual do sistema e para o dia selecionado.
   - Indicadores visuais de tarefas e eventos nos dias correspondentes (`•` para tarefas, `!` para atrasadas).
   - Painel lateral contextual com a lista detalhada de eventos e tarefas com vencimento no dia selecionado.
   - Atalho rápido `t` para retornar imediatamente para hoje.

3. **Lista Geral de Tarefas**:
   - Tabela organizada com status `[ ]` / `[✓]`, título, prioridade, data limite, situação amigável do prazo e categoria/projeto.
   - Atalho `Espaço` para alternar conclusão/reabertura da tarefa selecionada.
   - Atalho `d` para exclusão de tarefas.
   - Navegação intuitiva com setas (`↑` `↓`) ou `j`/`k`.

4. **Hierarquia de Projetos, Tarefas e Subtarefas (Árvore Expansível)**:
   - Estrutura hierárquica sem limite de profundidade (Projetos > Subprojetos > Tarefas > Subtarefas).
   - Barras de progresso visuais automáticas para cada projeto (`[████░░░░] 50% (2/4)`).
   - Prevenção ativa de ciclos na hierarquia (uma tarefa nunca pode ser ancestral de si mesma).
   - Conclusão individual e transparente de itens.

5. **Acompanhamento Dinâmico de Prazos**:
   - Rótulos amigáveis calculados dinamicamente: `Vence hoje`, `Falta 1 dia`, `Faltam X dias`, `Atrasada há X dias`, `Concluída`, `Sem prazo`.
   - Destaque com cores e alertas específicos para prazos vencidos ou urgentes.

6. **Visão Geral & Dashboard de Prazos**:
   - Painel em 4 quadrantes:
     - ⚠ **Atrasadas**: Tarefas vencidas com contagem de dias em atraso.
     - ⚡ **Para Hoje**: Tarefas que vencem na data atual.
     - 📅 **Próximos 7 Dias**: Tarefas da semana em ordem cronológica.
     - 📋 **Sem Prazo**: Tarefas gerais aguardando agendamento.

7. **Cadastro Rápido em Linguagem Natural (PT-BR)**:
   - Pressione `n` em qualquer tela para abrir o modal de cadastro rápido.
   - Interpretação inteligente de expressões:
     - `Estudar C amanhã` -> Título: *Estudar C*, Prazo: Amanhã
     - `Pagar internet dia 5` -> Título: *Pagar internet*, Prazo: Próximo dia 5
     - `Revisar álgebra sexta` -> Título: *Revisar álgebra*, Prazo: Próxima sexta-feira
     - `Prova de circuitos 12/11/2026` -> Título: *Prova de circuitos*, Prazo: 12/11/2026
     - `Comprar café` -> Título: *Comprar café*, Prazo: Sem data
   - Pré-visualização em tempo real da data identificada antes de confirmar.

8. **Persistência Confiável em SQLite**:
   - Tabelas normalizadas: `tasks`, `projects`, `events`, `schema_migrations`.
   - `PRAGMA foreign_keys = ON;` ativado.
   - Migrações automáticas incrementais (v1 → v3), com criação segura de colunas e índices.
   - Armazenamento em `~/.local/share/calma/calma.db` (ou `$XDG_DATA_HOME/calma/calma.db`).

9. **Agenda de Eventos Dedicada**:
   - Visualização própria (`5`) com lista cronológica de eventos.
   - Cadastro rápido em linguagem natural com data e horário opcionais (`E`, ou `n` dentro da agenda).
   - Edição (`e`) e exclusão (`d`) com confirmação segura.
   - Exemplos: `Reunião amanhã 14:30-16:00`, `Consulta dia 5 9h`, `Prova 12/11/2026`.
   - Horário de término opcional usando intervalo: `14:30-16:00` ou `14h-16h`.
   - Eventos também aparecem no calendário mensal e no painel do dia selecionado.

10. **Configurações Persistentes**:
   - Tela dedicada (`6`) com seções **Geral**, **Notificações** e **Aparência**.
   - Alternância de opções booleanas e edição de intervalos numéricos com validação.
   - Salvamento imediato em `~/.config/calma/config.toml` (ou `$XDG_CONFIG_HOME/calma/config.toml`).
   - Padrões conservadores: daemon, início automático e todas as categorias de notificação começam desativados.
   - Valores inválidos são normalizados com segurança; arquivos malformados nunca são sobrescritos automaticamente.

11. **Fundação do Daemon Independente**:
   - Separação entre o ciclo de vida da TUI e um processo de daemon que compartilha o mesmo SQLite.
   - Estrutura modular com serviço, agendador periódico e abstração de notificações.
   - Execução manual em primeiro plano com `calma daemon`; integração com systemd, IPC e notificações desktop ficam para fases futuras.

---

## ⌨️ Atalhos de Teclado

| Tecla | Ação |
|---|---|
| `1` ou `m` | Alternar para visualização de **Calendário Mensal** |
| `2` ou `l` | Alternar para visualização de **Lista de Tarefas** |
| `3` ou `p` | Alternar para visualização de **Hierarquia de Projetos** |
| `4` ou `v` | Alternar para visualização de **Visão Geral (Dashboard)** |
| `5` | Alternar para a **Agenda de Eventos** |
| `6` | Alternar para a tela de **Configurações** |
| `n` | Abrir **Cadastro Rápido** de nova tarefa (ou novo evento na Agenda) |
| `E` | Abrir **Cadastro Rápido de Evento** |
| `e` | Editar tarefa, projeto ou evento selecionado |
| `Espaço` | Marcar tarefa selecionada como concluída ou reabrir |
| `d` | Excluir item selecionado (tarefa, projeto ou evento) |
| `t` | Ir para **Hoje** |
| `↑` `↓` / `k` `j` | Navegar na lista, no dashboard ou na agenda de eventos |
| `j` / `k` | Selecionar tarefa no painel lateral do calendário |
| `←` `→` | Navegar pelos dias da semana no calendário |
| `PgUp` / `PgDn` | Mês anterior / Próximo mês no calendário |
| `?` | Abrir tela de ajuda com todos os atalhos |
| `Esc` | Fechar modal / Cancelar / Voltar |
| `q` | Sair do aplicativo |

Na tela de Configurações: `↑`/`↓` ou `j`/`k` navegam, `Enter`/`Espaço` alternam opções booleanas ou iniciam a edição de valores numéricos, `Esc` retorna à tela anterior e `q` encerra o Calma.

---

## 🛠 Requisitos de Sistema

- **Linux** (Arch Linux, Debian, Ubuntu, Fedora ou qualquer distribuição moderna).
- **Rust & Cargo** (versão 1.80+ recomendada).
- Biblioteca SQLite3 (`libsqlite3-dev` no Debian/Ubuntu, `sqlite` no Arch Linux).

---

## 📦 Compilação e Execução

### Executar em modo desenvolvimento:

```bash
cargo run
```

### Executar o daemon em primeiro plano:

```bash
cargo run -- daemon
```

O daemon só executa verificações quando o **monitoramento em segundo plano** está ativado na tela de Configurações. Ele é um processo separado da TUI e compartilha o mesmo banco SQLite.

### Executar a suíte de testes automatizados:

```bash
cargo test
```

### Gerar executável de produção otimizado:

```bash
cargo build --release
```

O binário final será gerado em:
```bash
target/release/calma
```

Você pode copiá-lo para seu `$PATH` (por exemplo, `~/.local/bin/calma` ou `/usr/local/bin/calma`):

```bash
cp target/release/calma ~/.local/bin/
```

---

## 📂 Onde os Dados são Armazenados

O Calma separa dados e configuração seguindo os padrões XDG:

**Dados (SQLite):**
- **Padrão**: `~/.local/share/calma/calma.db`
- Se `$XDG_DATA_HOME` estiver definida: `$XDG_DATA_HOME/calma/calma.db`

**Configuração (TOML):**
- **Padrão**: `~/.config/calma/config.toml`
- Se `$XDG_CONFIG_HOME` estiver definida: `$XDG_CONFIG_HOME/calma/config.toml`

O arquivo de configuração é criado automaticamente com valores padrão na primeira execução. A TUI e o daemon leem exatamente o mesmo arquivo.

### Backup e Restauração:

Para fazer backup dos seus dados:
```bash
cp ~/.local/share/calma/calma.db ~/backup_calma_$(date +%Y%m%d).db
```

Para restaurar um backup:
```bash
cp ~/backup_calma_*.db ~/.local/share/calma/calma.db
```

---

## 🏗 Arquitetura do Projeto

```text
calma/
├── Cargo.toml          # Dependências e metadados Rust
├── README.md           # Este manual
└── src/
    ├── main.rs         # Ponto de entrada, inicialização do terminal e loop principal
    ├── app.rs          # Estado da aplicação, modos de entrada e lógica de tela
    ├── config.rs       # Configuração TOML persistente e resolução de diretórios XDG
    ├── errors.rs       # Tratamento robusto de erros (AppError & AppResult)
    ├── input.rs        # Mapeamento e processamento de eventos do Crossterm
    ├── daemon/         # Daemon independente da TUI
    │   ├── mod.rs
    │   ├── service.rs       # Ciclo de vida do daemon
    │   ├── scheduler.rs     # Verificações periódicas de prazos e eventos
    │   └── notifications.rs # Abstração de notificações (sem entrega desktop ainda)
    ├── models/         # Entidades de domínio (Task, Project, Event)
    │   ├── mod.rs
    │   ├── task.rs
    │   ├── project.rs
    │   └── event.rs
    ├── db/             # Camada de persistência SQLite, migrações e testes
    │   └── mod.rs
    ├── services/       # Regras de negócio, manipulação de datas e parser PT-BR
    │   ├── mod.rs
    │   ├── calendar.rs
    │   └── date_parser.rs
    └── ui/             # Componentes visuais renderizados com Ratatui
        ├── mod.rs
        ├── theme.rs
        ├── header.rs
        ├── layout.rs
        ├── shortcuts.rs
        ├── calendar.rs
        ├── tasks.rs
        ├── hierarchy.rs
        ├── overview.rs
        ├── events.rs
        ├── settings.rs
        └── modal.rs
```

[English translation]

# Calma: Personal Time and Task Manager for the Terminal (TUI)

**Calma** is a personal manager for tasks, projects, events, and deadlines, running directly in the Linux terminal through a modern, minimal, and productivity-focused TUI (*Text User Interface*), designed to eliminate friction and unnecessary bureaucracy.

Built with **Rust**, **Ratatui**, **Crossterm**, and **SQLite**.

---

## 🎯 Goal

Help you organize your academic, personal, and financial life without requiring tedious planning or unnecessary bureaucracy. The goal is to make adding a task faster than putting it off.

- Tasks can exist with or without a due date.
- Important deadlines and events are always visible.
- Overdue items receive immediate visual emphasis and never silently disappear.
- Fully keyboard-driven terminal interface.
- Local data storage using SQLite, following Linux conventions (`XDG_DATA_HOME`).

---

## 🚀 Implemented Features

1. **Dynamic Brazilian Portuguese Header**:
   - Full weekday name (e.g., *Terça-feira*), complete date, and month/year, retrieved from the operating system in real time.
   - Real-time visual overview of task statistics: total tasks, completed tasks, tasks due today, and overdue tasks.
   - Indicator showing the currently active tab/view.

2. **Visual Monthly Calendar**:
   - 7 days of the week (starting on Monday).
   - Day and week navigation using the arrow keys (`←` `→` `↑` `↓`).
   - Month navigation using `PgUp` / `PgDn`.
   - Highlights both the current system date and the selected date.
   - Visual indicators for tasks and events on their respective dates (`•` for tasks, `!` for overdue tasks).
   - Contextual side panel displaying a detailed list of events and tasks due on the selected date.
   - Quick shortcut `t` to instantly return to today.

3. **General Task List**:
   - Organized table displaying status `[ ]` / `[✓]`, title, priority, due date, user-friendly deadline status, and category/project.
   - `Space` shortcut to toggle the selected task between completed and incomplete.
   - `d` shortcut to delete tasks.
   - Intuitive navigation using the arrow keys (`↑` `↓`) or `j`/`k`.

4. **Project, Task, and Subtask Hierarchy (Expandable Tree)**:
   - Hierarchical structure with unlimited nesting depth (Projects > Subprojects > Tasks > Subtasks).
   - Automatic visual progress bars for each project (`[████░░░░] 50% (2/4)`).
   - Active cycle prevention within the hierarchy (a task can never become its own ancestor).
   - Individual and transparent completion tracking for each item.

5. **Dynamic Deadline Tracking**:
   - Dynamically calculated, user-friendly labels: `Due today`, `1 day left`, `X days left`, `Overdue by X days`, `Completed`, `No deadline`.
   - Color highlights and specific alerts for overdue or urgent deadlines.

6. **Overview & Deadline Dashboard**:
   - Four-quadrant dashboard:
     - ⚠ **Overdue**: Overdue tasks with the number of days past their deadlines.
     - ⚡ **Due Today**: Tasks due on the current date.
     - 📅 **Next 7 Days**: Tasks due within the next week, sorted chronologically.
     - 📋 **No Deadline**: General tasks waiting to be scheduled.

7. **Quick Entry Using Natural Language (PT-BR)**:
   - Press `n` on any screen to open the quick-entry modal.
   - Intelligent interpretation of date expressions:
     - `Estudar C amanhã` -> Title: *Estudar C*, Due date: Tomorrow
     - `Pagar internet dia 5` -> Title: *Pagar internet*, Due date: Next occurrence of the 5th
     - `Revisar álgebra sexta` -> Title: *Revisar álgebra*, Due date: Next Friday
     - `Prova de circuitos 12/11/2026` -> Title: *Prova de circuitos*, Due date: 12/11/2026
     - `Comprar café` -> Title: *Comprar café*, Due date: None
   - Real-time preview of the detected date before confirmation.

8. **Reliable SQLite Persistence**:
   - Normalized tables: `tasks`, `projects`, `events`, `schema_migrations`.
   - `PRAGMA foreign_keys = ON;` enabled.
   - Automatic incremental migrations (v1 → v3), with safe column and index creation.
   - Data stored at `~/.local/share/calma/calma.db` (or `$XDG_DATA_HOME/calma/calma.db`).

9. **Dedicated Event Calendar**:
   - Dedicated view (`5`) displaying events in chronological order.
   - Quick entry using natural language, with optional date and time (`E`, or `n` while in the event calendar).
   - Editing (`e`) and deletion (`d`) with confirmation.
   - Examples: `Reunião amanhã 14:30-16:00`, `Consulta dia 5 9h`, `Prova 12/11/2026`.
   - Optional end time using a time range: `14:30-16:00` or `14h-16h`.
   - Events also appear in the monthly calendar and in the selected day's side panel.

10. **Persistent Settings**:
   - Dedicated screen (`6`) with **General**, **Notifications**, and **Appearance** sections.
   - Boolean toggles and validated numeric intervals.
   - Changes are saved immediately to `~/.config/calma/config.toml` (or `$XDG_CONFIG_HOME/calma/config.toml`).
   - Conservative defaults: daemon, automatic startup, and all notification categories are disabled.
   - Invalid values are safely normalized; malformed files are never overwritten automatically.

11. **Independent Daemon Foundation**:
   - TUI lifecycle is separate from a daemon process that shares the same SQLite database.
   - Modular service, periodic scheduler, and notification abstraction.
   - Manual foreground execution with `calma daemon`; systemd integration, IPC, and desktop delivery are future work.

---

## ⌨️ Keyboard Shortcuts

| Key | Action |
|---|---|
| `1` or `m` | Switch to **Monthly Calendar** view |
| `2` or `l` | Switch to **Task List** view |
| `3` or `p` | Switch to **Project Hierarchy** view |
| `4` or `v` | Switch to **Overview (Dashboard)** view |
| `5` | Switch to **Event Calendar** view |
| `n` | Open **Quick Entry** for a new task (or a new event in the Event Calendar) |
| `E` | Open **Quick Event Entry** |
| `e` | Edit the selected task, project, or event |
| `Space` | Mark the selected task as completed or reopen it |
| `d` | Delete the selected item (task, project, or event) |
| `t` | Jump to **Today** |
| `↑` `↓` / `k` `j` | Navigate through lists, the dashboard, or the event calendar |
| `j` / `k` | Select a task in the calendar's side panel |
| `←` `→` | Navigate between weekdays in the calendar |
| `PgUp` / `PgDn` | Previous month / Next month in the calendar |
| `?` | Open the help screen with all keyboard shortcuts |
| `Esc` | Close modal / Cancel / Go back |
| `q` | Quit the application |

In the Settings screen: `↑`/`↓` or `j`/`k` navigate, `Enter`/`Space` toggle booleans or start editing numeric values, `Esc` returns to the previous view, and `q` quits Calma.

---

## 🛠 System Requirements

- **Linux** (Arch Linux, Debian, Ubuntu, Fedora, or any modern distribution).
- **Rust & Cargo** (version 1.80+ recommended).
- SQLite3 library (`libsqlite3-dev` on Debian/Ubuntu, `sqlite` on Arch Linux).

---

## 📦 Building and Running

### Run in development mode:

```bash
cargo run
```

### Run the daemon in the foreground:

```bash
cargo run -- daemon
```

The daemon only performs checks when **background monitoring** is enabled in the Settings screen. It is a separate process from the TUI and shares the same SQLite database.

### Run the automated test suite:

```bash
cargo test
```

### Build an optimized release executable:

```bash
cargo build --release
```

The resulting binary will be generated at:

```bash
target/release/calma
```

You can copy it to a directory in your `$PATH` (for example, `~/.local/bin/calma` or `/usr/local/bin/calma`):

```bash
cp target/release/calma ~/.local/bin/
```

---

## 📂 Data Storage Location

Calma separates data and configuration following the XDG Base Directory Specification:

**Data (SQLite):**
- **Default**: `~/.local/share/calma/calma.db`
- If `$XDG_DATA_HOME` is set: `$XDG_DATA_HOME/calma/calma.db`

**Configuration (TOML):**
- **Default**: `~/.config/calma/config.toml`
- If `$XDG_CONFIG_HOME` is set: `$XDG_CONFIG_HOME/calma/config.toml`

The configuration file is created automatically with conservative defaults on first run. The TUI and daemon read the same file.

### Backup and Restore

To back up your data:

```bash
cp ~/.local/share/calma/calma.db ~/backup_calma_$(date +%Y%m%d).db
```

To restore a backup:

```bash
cp ~/backup_calma_*.db ~/.local/share/calma/calma.db
```

---

## 🏗 Project Architecture

```text
calma/
├── Cargo.toml          # Rust dependencies and metadata
├── README.md           # This documentation
└── src/
    ├── main.rs         # Entry point, terminal initialization, and main loop
    ├── app.rs          # Application state, input modes, and screen logic
    ├── config.rs       # Persistent TOML configuration and XDG resolution
    ├── errors.rs       # Robust error handling (AppError & AppResult)
    ├── input.rs        # Crossterm event mapping and processing
    ├── daemon/         # TUI-independent daemon foundation
    │   ├── mod.rs
    │   ├── service.rs       # Daemon lifecycle
    │   ├── scheduler.rs     # Periodic deadline/event checks
    │   └── notifications.rs # Notification abstraction (no desktop delivery yet)
    ├── models/         # Domain entities (Task, Project, Event)
    │   ├── mod.rs
    │   ├── task.rs
    │   ├── project.rs
    │   └── event.rs
    ├── db/             # SQLite persistence layer, migrations, and tests
    │   └── mod.rs
    ├── services/       # Business logic, date handling, and PT-BR parser
    │   ├── mod.rs
    │   ├── calendar.rs
    │   └── date_parser.rs
    └── ui/             # Visual components rendered with Ratatui
        ├── mod.rs
        ├── theme.rs
        ├── header.rs
        ├── layout.rs
        ├── shortcuts.rs
        ├── calendar.rs
        ├── tasks.rs
        ├── hierarchy.rs
        ├── overview.rs
        ├── events.rs
        ├── settings.rs
        └── modal.rs
```
