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
   - Tela dedicada (`6`) com seções **Geral**, **Notificações**, **Ambiente Gráfico** e **Aparência**.
   - Alternância de opções booleanas e edição de intervalos numéricos com validação.
   - Salvamento imediato em `~/.config/calma/config.toml` (ou `$XDG_CONFIG_HOME/calma/config.toml`).
   - Padrões conservadores: daemon, início automático e todas as categorias de notificação começam desativados.
   - Valores inválidos são normalizados com segurança; arquivos malformados nunca são sobrescritos automaticamente.
   - A seção **Ambiente Gráfico** é somente leitura e mostra desktop, sessão, backend, disponibilidade do serviço e diagnóstico; `r` atualiza a detecção.

11. **Daemon Independente e Notificações Nativas**:
   - Separação entre o ciclo de vida da TUI e um processo de daemon que compartilha o mesmo SQLite.
   - Estrutura modular com serviço, agendador periódico e abstração de notificações.
   - Entrega nativa pelo barramento D-Bus da sessão via `org.freedesktop.Notifications`, com suporte verificado em KDE Plasma.
   - Avisos únicos por tarefa/evento no dia e um aviso adicional dentro da antecedência configurada para eventos com horário.
   - Execução manual em primeiro plano com `calma daemon`; integração com systemd e IPC continuam fora do escopo.

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
| `r` | Atualizar o diagnóstico do ambiente gráfico nas Configurações |
| `?` | Abrir tela de ajuda com todos os atalhos |
| `Esc` | Fechar modal / Cancelar / Voltar |
| `q` | Sair do aplicativo |

Na tela de Configurações: `↑`/`↓` ou `j`/`k` navegam, `Enter`/`Espaço` alternam opções booleanas ou iniciam a edição de valores numéricos, `r` atualiza a detecção do ambiente e a disponibilidade do serviço, `Esc` retorna à tela anterior e `q` encerra o Calma.

---

## 🛠 Requisitos de Sistema

- **Linux** (Arch Linux, Debian, Ubuntu, Fedora ou qualquer distribuição moderna).
- **Rust & Cargo** (versão 1.80+ recomendada).
- Biblioteca SQLite3 (`libsqlite3-dev` no Debian/Ubuntu, `sqlite` no Arch Linux).
- Para notificações desktop: sessão gráfica com barramento D-Bus de sessão e um serviço `org.freedesktop.Notifications` (KDE Plasma é o ambiente testado).

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

Para entregar notificações, o daemon precisa herdar o ambiente da sessão gráfica, em especial `DBUS_SESSION_BUS_ADDRESS` e, quando aplicável, `WAYLAND_DISPLAY` ou `DISPLAY`. Iniciar o daemon manualmente a partir de um terminal da própria sessão atende a esse requisito. O Calma **não** implementa integração com systemd nesta fase; um serviço de usuário iniciado fora da sessão pode não ter acesso ao barramento correto.

Se nenhum backend estiver disponível, o daemon continua executando normalmente e registra o diagnóstico em `stderr`. Notificações são deduplicadas em memória: cada tarefa ou evento gera no máximo um aviso por marco (dia do item e antecedência do evento) durante a execução do daemon. Reiniciar o processo reinicia esse estado.

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
    │   ├── service.rs       # Ciclo de vida e seleção do backend de notificação
    │   ├── scheduler.rs     # Verificações periódicas de prazos e eventos
    │   └── notifications.rs # Geração de notificações e reexportação da abstração
    ├── desktop/        # Detecção de ambiente e entrega nativa
    │   ├── mod.rs
    │   ├── detection.rs     # DesktopEnvironment, SessionType e DesktopSession
    │   ├── environment.rs   # Snapshot injetável das variáveis de ambiente
    │   └── notifications/
    │       ├── mod.rs       # Seleção de backend e diagnóstico
    │       ├── notifier.rs  # Notification, Notifier e NotificationError
    │       ├── freedesktop.rs # D-Bus org.freedesktop.Notifications (zbus)
    │       ├── kde.rs       # Invólucro fino sobre o backend Freedesktop
    │       └── noop.rs      # Fallback sem efeitos colaterais
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

## 🔔 Notificações Desktop e Detecção de Ambiente

### Ambientes suportados na detecção

A detecção lê múltiplas variáveis (`XDG_CURRENT_DESKTOP`, `XDG_SESSION_DESKTOP`, `DESKTOP_SESSION`, `KDE_SESSION_VERSION`, `XDG_SESSION_TYPE`, `WAYLAND_DISPLAY`, `DISPLAY` e `DBUS_SESSION_BUS_ADDRESS`) e reconhece:

- KDE Plasma
- GNOME
- XFCE
- Cinnamon
- MATE
- LXQt
- i3
- Sway
- ambiente desconhecido (sessão gráfica sem identificação)
- headless (sem sessão gráfica)

A ordem de prioridade é `XDG_CURRENT_DESKTOP`, `XDG_SESSION_DESKTOP` e `DESKTOP_SESSION`. Se a fonte prioritária contiver apenas valores desconhecidos, as seguintes são consultadas. `KDE_SESSION_VERSION` funciona como último indício de KDE. A presença de D-Bus, sozinha, não caracteriza uma sessão gráfica.

**Detectar o desktop é diferente de o serviço de notificações estar disponível.** O Calma não considera o KDE disponível apenas porque pacotes KDE estão instalados; a disponibilidade é verificada em tempo de execução pela chamada D-Bus `GetServerInformation`.

### Backends de notificação

- **KDE / Freedesktop D-Bus**: alvo verificado, usado no KDE Plasma por meio da especificação `org.freedesktop.Notifications`.
- **Freedesktop D-Bus**: mesmo protocolo, sem duplicação de implementação, para GNOME, XFCE, Cinnamon, MATE, LXQt e outros ambientes compatíveis.
- **Noop**: fallback seguro quando não há ambiente gráfico ou serviço disponível.

Valores aceitos em `[notifications.desktop] backend`:

- `auto`: seleciona KDE no Plasma e Freedesktop nos demais ambientes, com fallback para Noop.
- `kde`: força o backend KDE/Freedesktop.
- `freedesktop`: força o backend Freedesktop genérico.
- `noop`: desativa a entrega sem remover as preferências.
- Qualquer outro valor gera diagnóstico e cai de volta em `auto`.

A entrega usa `zbus` com API bloqueante, uma conexão reutilizada, tempo limite de método (2 segundos por padrão) e sem runtime assíncrono próprio. As chamadas usam `Notify` com nome de aplicativo `Calma`. Não há chamada a `notify-send` nem construção manual de mensagens D-Bus.

### Configuração

```toml
[notifications]
enabled = false
upcoming_tasks = false
overdue_tasks = false
upcoming_events = false
advance_minutes = 60

[notifications.desktop]
backend = "auto"
```

Configurações antigas sem a seção `[notifications.desktop]` continuam válidas e recebem `backend = "auto"`. O resultado da detecção é sempre de runtime e nunca é gravado no TOML.

### Comportamento dos avisos

- Tarefas não concluídas com `due_date` ou `scheduled_date` igual ao dia atual geram um aviso uma única vez.
- Tarefas atrasadas geram no máximo um aviso por dia.
- Eventos do dia geram um aviso único no dia; eventos já encerrados são ignorados.
- Eventos com horário de início geram um segundo aviso único quando faltam `advance_minutes` minutos ou menos para começar.
- Eventos de dia inteiro (`start_time` ausente) recebem apenas o aviso do dia.
- A identidade do aviso inclui o tipo, o item, a data e o marco (dia ou antecedência), permitindo que o mesmo evento receba os dois avisos previstos sem repetição.
- A deduplicação é feita em memória e reiniciada quando o dia muda. Reiniciar o daemon pode repetir um aviso já enviado.

### Tela de Configurações

A seção **AMBIENTE GRÁFICO** mostra, em modo somente leitura:

```text
Desktop detectado: KDE Plasma
Sessão: Wayland
Backend: KDE / Freedesktop D-Bus
Serviço de notificações: Disponível
Entrega de notificações: Ativada
Diagnóstico: Serviço de notificações disponível via KDE / Freedesktop D-Bus.
```

Quando não há ambiente gráfico, os campos indicam `Desktop detectado: Nenhum`, `Sessão: Headless`, `Backend: Nenhum` e `Serviço de notificações: Indisponível`. A entrega também é mostrada como desativada quando a configuração assim determina, evitando estados de sucesso enganosos. `r` atualiza o diagnóstico sem reiniciar a TUI.

### Verificação manual em uma sessão real

Em uma sessão KDE Plasma com D-Bus ativo:

```bash
cargo test manual_kde_notification_smoke -- --ignored --nocapture
```

Esse teste envia uma notificação real. A suíte padrão (`cargo test`) não depende de sessão gráfica, de D-Bus ou de KDE; todos os testes de detecção e de entrega usam dados e transportes injetados.

### Limitações atuais

- A entrega foi verificada automaticamente via D-Bus real apenas no KDE Plasma deste ambiente de desenvolvimento. GNOME, XFCE e os demais ambientes são suportados pela arquitetura e pelo protocolo Freedesktop, mas não foram verificados manualmente nesta entrega.
- Não há backend para Windows ou macOS.
- Não há integração com systemd, IPC ou rede.
- A deduplicação é apenas em memória: reiniciar o daemon pode repetir um aviso já enviado no mesmo dia.
- O aviso de antecedência é emitido na primeira verificação que encontrar o evento dentro da janela configurada; com `poll_interval_secs = 1800`, por exemplo, ele pode chegar um pouco menos de 60 minutos antes.
- A seleção do backend é feita uma vez na inicialização do daemon; se o serviço de notificações só ficar disponível depois, reinicie o daemon.
- A detecção depende de variáveis de ambiente; sessões iniciadas por SSH, contêineres ou serviços de usuário podem apresentar valores incompletos.
- Notificações são entregues apenas pelo daemon; a TUI não envia notificações.

### Como adicionar um novo backend

1. Implemente `Notifier` em `src/desktop/notifications/`.
2. Adicione o novo valor em `BackendKind` e na fábrica `NotifierFactory`.
3. Registre a preferência em `BackendPreference::parse`.
4. Adicione testes com transporte/fábrica falsos.
5. O daemon, o agendador e a geração de notificações não precisam ser alterados.

Para ambientes Linux que já falam Freedesktop, reutilize o backend existente em vez de criar uma implementação duplicada.

---

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
   - Dedicated screen (`6`) with **General**, **Notifications**, **Desktop Environment**, and **Appearance** sections.
   - Boolean toggles and validated numeric intervals.
   - Changes are saved immediately to `~/.config/calma/config.toml` (or `$XDG_CONFIG_HOME/calma/config.toml`).
   - Conservative defaults: daemon, automatic startup, and all notification categories are disabled.
   - Invalid values are safely normalized; malformed files are never overwritten automatically.
   - The **Desktop Environment** section is read-only and shows the detected desktop, session type, backend, service availability, and diagnostic message; `r` refreshes detection.

11. **Independent Daemon and Native Notifications**:
   - TUI lifecycle is separate from a daemon process that shares the same SQLite database.
   - Modular service, periodic scheduler, and notification abstraction.
   - Native delivery through the session D-Bus using `org.freedesktop.Notifications`, with verified support on KDE Plasma.
   - One notification per task/event on its day, plus one additional notification within the configured lead time for timed events.
   - Manual foreground execution with `calma daemon`; systemd integration and IPC remain out of scope.

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
| `r` | Refresh the desktop environment diagnostics in Settings |
| `?` | Open the help screen with all keyboard shortcuts |
| `Esc` | Close modal / Cancel / Go back |
| `q` | Quit the application |

In the Settings screen: `↑`/`↓` or `j`/`k` navigate, `Enter`/`Space` toggle booleans or start editing numeric values, `r` refreshes environment detection and service availability, `Esc` returns to the previous view, and `q` quits Calma.

---

## 🛠 System Requirements

- **Linux** (Arch Linux, Debian, Ubuntu, Fedora, or any modern distribution).
- **Rust & Cargo** (version 1.80+ recommended).
- SQLite3 library (`libsqlite3-dev` on Debian/Ubuntu, `sqlite` on Arch Linux).
- For desktop notifications: a graphical session with a session D-Bus and an `org.freedesktop.Notifications` service (KDE Plasma is the tested environment).

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

To deliver notifications, the daemon must inherit the graphical session environment, especially `DBUS_SESSION_BUS_ADDRESS` and, when applicable, `WAYLAND_DISPLAY` or `DISPLAY`. Starting the daemon manually from a terminal inside the session satisfies this requirement. Calma does **not** implement systemd integration at this stage; a user service started outside the session may not have access to the correct bus.

If no backend is available, the daemon keeps running normally and writes the diagnostic to `stderr`. Notifications are deduplicated in memory: each task or event produces at most one notification per milestone (day-of and event lead time) while the daemon runs. Restarting the process resets that state.

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

---

## 🔔 Desktop Notifications and Environment Detection

### Environments supported by detection

Detection reads multiple variables (`XDG_CURRENT_DESKTOP`, `XDG_SESSION_DESKTOP`, `DESKTOP_SESSION`, `KDE_SESSION_VERSION`, `XDG_SESSION_TYPE`, `WAYLAND_DISPLAY`, `DISPLAY`, and `DBUS_SESSION_BUS_ADDRESS`) and recognizes:

- KDE Plasma
- GNOME
- XFCE
- Cinnamon
- MATE
- LXQt
- i3
- Sway
- unknown environment (graphical session without identification)
- headless (no graphical session)

Priority is `XDG_CURRENT_DESKTOP`, then `XDG_SESSION_DESKTOP`, then `DESKTOP_SESSION`. If the priority source contains only unknown values, the following sources are still consulted. `KDE_SESSION_VERSION` acts as a last KDE hint. The presence of D-Bus alone does not indicate a graphical session.

**Detecting the desktop is different from knowing whether the notification service is available.** Calma does not consider KDE available merely because KDE packages are installed; availability is checked at runtime through the `GetServerInformation` D-Bus call.

### Notification backends

- **KDE / Freedesktop D-Bus**: verified target, used on KDE Plasma through the `org.freedesktop.Notifications` specification.
- **Freedesktop D-Bus**: the same protocol, without duplicated implementation, for GNOME, XFCE, Cinnamon, MATE, LXQt, and other compatible environments.
- **Noop**: safe fallback when there is no graphical environment or available service.

Accepted values for `[notifications.desktop] backend`:

- `auto`: selects KDE on Plasma and Freedesktop on other environments, falling back to Noop.
- `kde`: forces the KDE/Freedesktop backend.
- `freedesktop`: forces the generic Freedesktop backend.
- `noop`: disables delivery without removing preferences.
- Any other value produces a diagnostic and falls back to `auto`.

Delivery uses `zbus` with its blocking API, a reused connection, a method timeout (2 seconds by default), and no proprietary async runtime. Calls use `Notify` with the application name `Calma`. It does not shell out to `notify-send` and does not hand-build D-Bus messages.

### Configuration

```toml
[notifications]
enabled = false
upcoming_tasks = false
overdue_tasks = false
upcoming_events = false
advance_minutes = 60

[notifications.desktop]
backend = "auto"
```

Older configurations without `[notifications.desktop]` remain valid and receive `backend = "auto"`. Detection results are always runtime-only and are never written to TOML.

### Notification behavior

- Non-completed tasks whose `due_date` or `scheduled_date` is today generate one notification.
- Overdue tasks generate at most one notification per day.
- Events occurring today generate one notification that day; events that already ended are ignored.
- Timed events generate a second, distinct notification once the event starts within `advance_minutes` minutes.
- All-day events (`start_time` absent) receive only the day-of notification.
- The notification identity includes the kind, item, date, and milestone (day-of or lead time), so the same event can receive both expected notifications without repetition.
- Deduplication is in memory and resets when the day changes. Restarting the daemon may repeat a notification already sent.

### Settings screen

The **AMBIENTE GRÁFICO** (Desktop Environment) section shows, read-only:

```text
Desktop detectado: KDE Plasma
Sessão: Wayland
Backend: KDE / Freedesktop D-Bus
Serviço de notificações: Disponível
Entrega de notificações: Ativada
Diagnóstico: Serviço de notificações disponível via KDE / Freedesktop D-Bus.
```

When there is no graphical environment, the fields show `Desktop detectado: Nenhum`, `Sessão: Headless`, `Backend: Nenhum`, and `Serviço de notificações: Indisponível`. Delivery also shows as disabled when configuration says so, avoiding misleading success states. `r` refreshes the diagnostics without restarting the TUI.

### Manual verification in a real session

On a KDE Plasma session with D-Bus active:

```bash
cargo test manual_kde_notification_smoke -- --ignored --nocapture
```

This test sends a real notification. The default suite (`cargo test`) does not depend on a graphical session, D-Bus, or KDE; all detection and delivery tests use injected data and transports.

### Current limitations

- Delivery was verified automatically through real D-Bus only on KDE Plasma in this development environment. GNOME, XFCE, and other environments are supported by the architecture and Freedesktop protocol, but were not manually verified in this delivery.
- There is no Windows or macOS backend.
- There is no systemd integration, IPC, or networking.
- Deduplication is memory-only: restarting the daemon may repeat a notification already sent that day.
- The lead-time notification is emitted on the first check that finds the event inside the configured window; with `poll_interval_secs = 1800`, for example, it may arrive slightly less than 60 minutes before the event.
- The backend is selected once when the daemon starts; if the notification service only becomes available later, restart the daemon.
- Detection depends on environment variables; sessions started via SSH, containers, or user services may expose incomplete values.
- Notifications are delivered only by the daemon; the TUI does not send notifications.

### Adding a new backend

1. Implement `Notifier` under `src/desktop/notifications/`.
2. Add the new value to `BackendKind` and to the `NotifierFactory`.
3. Register the preference in `BackendPreference::parse`.
4. Add tests with fake transports/factories.
5. The daemon, scheduler, and notification generation do not need changes.

For Linux environments that already speak Freedesktop, reuse the existing backend instead of creating a duplicated implementation.

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
    │   ├── service.rs       # Lifecycle and notification backend selection
    │   ├── scheduler.rs     # Periodic deadline/event checks
    │   └── notifications.rs # Notification generation and abstraction re-exports
    ├── desktop/        # Environment detection and native delivery
    │   ├── mod.rs
    │   ├── detection.rs     # DesktopEnvironment, SessionType, and DesktopSession
    │   ├── environment.rs   # Injectable environment snapshot
    │   └── notifications/
    │       ├── mod.rs       # Backend selection and diagnostics
    │       ├── notifier.rs  # Notification, Notifier, and NotificationError
    │       ├── freedesktop.rs # org.freedesktop.Notifications over D-Bus (zbus)
    │       ├── kde.rs       # Thin wrapper over the Freedesktop backend
    │       └── noop.rs      # Side-effect-free fallback
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
