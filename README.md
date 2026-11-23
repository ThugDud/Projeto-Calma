# Calma: Gerenciador Pessoal de Tempo e Tarefas em TUI

O **Calma** é um gerenciador pessoal de tarefas, projetos, compromissos e datas, executado diretamente no terminal Linux por meio de uma interface TUI (*Text User Interface*) moderna, sóbria e focada em produtividade sem atritos ou burocracia.

Desenvolvido em **Rust** com **Ratatui**, **Crossterm** e **SQLite**.

---

## 🎯 Objetivo

Ajudar você a organizar a vida acadêmica, pessoal e financeira sem exigir planejamento burocrático. O objetivo é que registrar uma tarefa seja mais rápido do que deixá-la para depois.

- Tarefas podem existir com ou sem data definida.
- Tarefas podem se repetir diariamente ou em dias específicos da semana, com horário opcional.
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
   - Indicadores visuais nos dias: `•` tarefas não recorrentes, `◆` apenas eventos, `◉` tarefas e eventos, `!` tarefas atrasadas.
   - Tarefas recorrentes não aparecem no calendário; elas têm uma seção própria organizada por dia da semana.
   - Painel lateral contextual com a lista detalhada de eventos e tarefas com vencimento no dia selecionado.
   - Atalho rápido `t` para retornar imediatamente para hoje.

3. **Lista Geral de Tarefas**:
   - Tabela organizada com status `[ ]` / `[✓]`, título, prioridade, data limite, situação amigável do prazo e categoria/projeto.
   - Tarefas recorrentes aparecem com `↻` e a regra (diária ou semanal) na coluna de data.
   - Atalho `Espaço` para alternar conclusão/reabertura da tarefa selecionada.
   - Em tarefas recorrentes, a conclusão vale apenas para a ocorrência do dia (ou da data selecionada no calendário).
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
   - Recorrência no próprio cadastro rápido:
     - `Academia diariamente 7h` -> repete todos os dias às 07:00
     - `Revisar seg,qua,sex 09:30` -> repete segunda, quarta e sexta às 09:30
     - `Feira toda segunda` -> repete toda segunda-feira
   - A conclusão de uma tarefa recorrente vale apenas para a ocorrência do dia (ou da data selecionada no calendário), sem desativar a repetição.
   - Pré-visualização em tempo real da data identificada antes de confirmar.

8. **Persistência Confiável em SQLite**:
   - Tabelas normalizadas: `tasks`, `projects`, `events`, `task_completions`, `schema_migrations`.
   - `PRAGMA foreign_keys = ON;` ativado.
   - Migrações automáticas incrementais (v1 → v4), com criação segura de colunas, índices e tabelas.
   - Armazenamento em `~/.local/share/calma/calma.db` (ou `$XDG_DATA_HOME/calma/calma.db`).

9. **Agenda de Eventos Dedicada**:
   - Visualização própria (`5`) com lista cronológica de eventos.
   - Cadastro rápido em linguagem natural com data e horário opcionais (`E`, ou `n` dentro da agenda).
   - Edição (`e`) e exclusão (`d`) com confirmação segura.
   - Exemplos: `Reunião amanhã 14:30-16:00`, `Consulta dia 5 9h`, `Prova 12/11/2026`.
   - Horário de término opcional usando intervalo: `14:30-16:00` ou `14h-16h`.
   - Eventos também aparecem no calendário mensal e no painel do dia selecionado.

10. **Tarefas Recorrentes por Dia da Semana**:
   - Seção dedicada (`7`) com sete colunas: segunda a domingo.
   - Cada tarefa recorrente aparece na coluna do dia em que ocorre; tarefas diárias aparecem em todas as colunas.
   - O dia atual é destacado visualmente e a semana exibida é a semana corrente.
   - Navegação por colunas com `←`/`→` e seleção de tarefas com `↑`/`↓`.
   - `Espaço` conclui apenas a ocorrência daquele dia; `e` edita e `d` exclui a série.
   - Tarefas concluídas aparecem com `[✓]` e o horário é exibido quando definido.

11. **Configurações Persistentes**:
   - Tela dedicada (`6`) com seções **Geral**, **Notificações**, **Ambiente Gráfico** e **Aparência**.
   - Alternância de opções booleanas e edição de intervalos numéricos com validação.
   - Salvamento imediato em `~/.config/calma/config.toml` (ou `$XDG_CONFIG_HOME/calma/config.toml`).
   - Padrões conservadores: daemon, início automático e todas as categorias de notificação começam desativados.
   - Valores inválidos são normalizados com segurança; arquivos malformados nunca são sobrescritos automaticamente.
   - A seção **Ambiente Gráfico** é somente leitura e mostra desktop, sessão, backend, disponibilidade do serviço e diagnóstico; `r` atualiza a detecção.

12. **Daemon Independente e Notificações Nativas**:
   - Separação entre o ciclo de vida da TUI e um processo de daemon que compartilha o mesmo SQLite.
   - Estrutura modular com serviço, agendador periódico e abstração de notificações.
   - Entrega nativa pelo barramento D-Bus da sessão via `org.freedesktop.Notifications`, com suporte verificado em KDE Plasma.
   - Avisos únicos por tarefa/evento no dia e um aviso adicional dentro da antecedência configurada para eventos com horário.
   - Inicialização automática opcional via serviço systemd de usuário criado e habilitado pela própria TUI, sem edição manual de arquivos.
   - Ícone na bandeja do KDE com três estados (good/attention/late vibes) e clique para abrir a TUI no terminal padrão.
   - Execução manual em primeiro plano com `calma daemon`; IPC continua fora do escopo.

---

## ⌨️ Atalhos de Teclado

| Tecla | Ação |
|---|---|
| `Tab` | Abrir o **menu de seções** (navegue com `↑`/`↓` e confirme com `Enter`) |
| `1` ou `m` | Alternar para visualização de **Calendário Mensal** |
| `2` ou `l` | Alternar para visualização de **Lista de Tarefas** |
| `3` ou `h` | Alternar para visualização de **Hierarquia de Projetos** |
| `4` ou `v` | Alternar para visualização de **Visão Geral (Dashboard)** |
| `5` | Alternar para a **Agenda de Eventos** |
| `6` | Alternar para a tela de **Configurações** |
| `7` | Alternar para **Tarefas Recorrentes** (colunas por dia da semana) |
| `n` | Abrir **Cadastro Rápido** de nova tarefa (ou novo evento na Agenda) |
| `E` | Abrir **Cadastro Rápido de Evento** |
| `e` | Editar tarefa, projeto ou evento selecionado |
| `Espaço` | Marcar tarefa selecionada como concluída ou reabrir |
| `d` | Excluir item selecionado (tarefa, projeto ou evento) |
| `t` | Ir para **Hoje** |
| `x` | **Exportar** projetos, tarefas e eventos para um arquivo JSON |
| `i` | **Importar** dados de um arquivo JSON (adicionar ou sobrescrever) |
| `↑` `↓` / `k` `j` | Navegar na lista, no dashboard ou na agenda de eventos |
| `j` / `k` | Selecionar tarefa no painel lateral do calendário |
| `←` `→` | Navegar pelos dias da semana no calendário |
| `PgUp` / `PgDn` | Mês anterior / Próximo mês no calendário |
| `r` | Atualizar o diagnóstico do ambiente gráfico nas Configurações |
| `?` | Abrir tela de ajuda com todos os atalhos |
| `Esc` | Fechar modal / Cancelar / Voltar |
| `q` | Sair do aplicativo |

Na tela de Configurações: `↑`/`↓` ou `j`/`k` navegam, `Enter`/`Espaço` alternam opções booleanas ou iniciam a edição de valores numéricos, `r` atualiza a detecção do ambiente e a disponibilidade do serviço, `x` exporta e `i` importa dados, `Esc` retorna à tela anterior e `q` encerra o Calma.

No formulário de edição de tarefa, `Tab`/`Shift+Tab` alternam entre Título, Prazo, Horário, Repetição, Dias, Prioridade e Projeto. No campo Repetição, use `0` (nenhuma), `1` (diária) ou `2` (semanal); no campo Dias, `1` a `7` alternam segunda a domingo; no campo Prioridade, `1` a `4` escolhem baixa, normal, alta e urgente.

---

## 🛠 Requisitos de Sistema

- **Linux** (Arch Linux, Debian, Ubuntu, Fedora ou qualquer distribuição moderna).
- **Rust & Cargo** (versão 1.80+ recomendada).
- Biblioteca SQLite3 (`libsqlite3-dev` no Debian/Ubuntu, `sqlite` no Arch Linux).
- Para notificações desktop: sessão gráfica com barramento D-Bus de sessão e um serviço `org.freedesktop.Notifications` (KDE Plasma é o ambiente testado).
- Para inicialização automática: systemd de usuário (`systemctl --user`). Outros gerenciadores ainda não são suportados.
- Para o ícone na bandeja: KDE Plasma com `org.kde.StatusNotifierWatcher` (o ícone aparece na área de serviços em segundo plano) e um terminal instalado. Nesta versão apenas o backend KDE está implementado.

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

Para entregar notificações, o daemon precisa herdar o ambiente da sessão gráfica, em especial `DBUS_SESSION_BUS_ADDRESS` e, quando aplicável, `WAYLAND_DISPLAY` ou `DISPLAY`. O serviço systemd de usuário é iniciado pelo gerenciador de sessão do próprio usuário e normalmente herda essas variáveis. Um serviço iniciado fora da sessão (por SSH, contêiner ou com ambiente incompleto) pode não ter acesso ao barramento correto. A TUI instala e habilita esse serviço automaticamente; não é necessário criar arquivos `.service` manualmente.

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

#### Pelo próprio Calma (recomendado)

Pressione `x` em qualquer tela (inclusive nas Configurações) para exportar projetos, tarefas e eventos para um arquivo JSON. O caminho padrão sugerido é `~/calma-backup-AAAAMMDD-HHMMSS.json`, mas pode ser editado antes de confirmar.

O arquivo usa o formato versionado `calma-backup` (versão `1`) e contém `projects`, `tasks`, `events`, `task_completions` e `exported_at`. A recorrência e o horário das tarefas também são preservados. Arquivos gerados por versões anteriores, sem `task_completions`, continuam válidos. A gravação é atômica: o conteúdo é escrito em um arquivo temporário e depois renomeado.

Pressione `i` para importar. Informe o caminho do JSON e escolha um dos modos:

- **Importar e adicionar**: mantém os dados atuais e insere apenas itens que ainda não existem. A comparação ignora IDs, datas de criação, `sort_order` e `completed_at`, e normaliza maiúsculas/minúsculas e espaços para evitar duplicatas. Projetos, tarefas e eventos duplicados são ignorados.
- **Importar e sobrescrever**: apaga todos os projetos, tarefas e eventos atuais antes de aplicar o arquivo. Esta opção é destrutiva e exige confirmação explícita; o modo padrão é sempre o de adicionar.

A importação é transacional e validada antes de qualquer alteração: formato, versão, IDs, referências entre projetos/tarefas e ciclos na hierarquia. Se qualquer item for inválido, nada é alterado no banco.

#### Cópia manual do banco SQLite

Para fazer backup do banco completo:
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
    │   ├── date_parser.rs
    │   └── portability.rs   # Exportação/importação JSON versionada do estado
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
[general]
start_with_system = false
background_monitoring = false
poll_interval_secs = 60

[notifications]
enabled = false
upcoming_tasks = false
overdue_tasks = false
upcoming_events = false
advance_minutes = 60

[notifications.desktop]
backend = "auto"

[tray]
enabled = true
backend = "auto"
terminal = ""
```

Configurações antigas sem a seção `[notifications.desktop]` continuam válidas e recebem `backend = "auto"`. Configurações sem `[tray]` recebem `enabled = true`, `backend = "auto"` e `terminal = ""`. O resultado da detecção é sempre de runtime e nunca é gravado no TOML.

### Comportamento dos avisos

- Tarefas não concluídas com `due_date` ou `scheduled_date` igual ao dia atual geram um aviso uma única vez.
- Tarefas recorrentes geram o aviso do dia apenas quando a regra indica ocorrência na data atual; a ocorrência concluída não gera aviso.
- Tarefas de hoje com horário definido geram um segundo aviso único quando faltam `advance_minutes` minutos ou menos para começar.
- Tarefas atrasadas geram no máximo um aviso por dia.
- Eventos do dia geram um aviso único no dia; eventos já encerrados são ignorados.
- Eventos com horário de início geram um segundo aviso único quando faltam `advance_minutes` minutos ou menos para começar.
- Eventos de dia inteiro (`start_time` ausente) recebem apenas o aviso do dia.
- A identidade do aviso inclui o tipo, o item, a data e o marco (dia ou antecedência), permitindo que a mesma tarefa/evento receba os dois avisos previstos sem repetição.
- A deduplicação é feita em memória e reiniciada quando o dia muda. Reiniciar o daemon pode repetir um aviso já enviado.

### Inicialização automática do daemon (systemd)

A opção **Iniciar com o sistema**, em **Configurações > Geral**, cuida de toda a integração sem exigir que o usuário crie ou edite arquivos de serviço:

- Cria `$XDG_CONFIG_HOME/systemd/user/calma-daemon.service` (ou `~/.config/systemd/user/calma-daemon.service`).
- Gera um `ExecStart` apontando para o executável atual do Calma seguido de `daemon`.
- Executa `systemctl --user daemon-reload` e `systemctl --user enable --now calma-daemon.service`.
- Ao desativar a opção, executa `systemctl --user disable --now calma-daemon.service`, remove o arquivo da unidade e recarrega o systemd.
- Ativar a inicialização automática também liga o **monitoramento em segundo plano**, para que o daemon tenha trabalho ao iniciar.
- Quando a TUI abre com a preferência já ativa, a unidade é reconciliada com o executável atual, cobrindo atualizações e reinstalações do Calma.
- Se a operação falhar, a preferência é revertida e a tela de configurações mostra o motivo.

O backend é abstraído por `ServiceBackend`/`AutostartManager` em `src/services/autostart.rs`. A versão atual implementa apenas o systemd de usuário; outros gerenciadores podem ser adicionados implementando o trait e registrando o novo backend na ordem de preferência, sem alterar a TUI.

### Ícone na bandeja do sistema (KDE)

Quando o daemon está em execução e há um ambiente compatível, o Calma registra um **StatusNotifierItem** na bandeja do KDE (a área de serviços em segundo plano, no canto inferior direito). O ícone reflete o estado efetivo das tarefas:

| Ícone | Estado | Critério |
|---|---|---|
| good vibes | Tudo em dia | Nenhuma tarefa atrasada ou pendente para hoje |
| attention vibes | Tarefas para hoje | Existe tarefa não concluída que ocorre hoje |
| late vibes | Tarefas atrasadas | Existe tarefa não concluída com prazo anterior a hoje |

As regras são as mesmas da TUI e do agendador: tarefas recorrentes contam apenas quando ocorrem na data e a conclusão é verificada por ocorrência. `late vibes` tem prioridade sobre `attention vibes`. O tooltip mostra as contagens e o menu da bandeja oferece **Abrir Calma** e **Sair do daemon**.

Um clique no ícone (ou em **Abrir Calma**) abre a TUI em um processo separado, usando o terminal padrão detectado nesta ordem:

1. `terminal` da seção `[tray]` da configuração, quando preenchido;
2. variável de ambiente `TERMINAL`;
3. preferência de terminal do KDE Plasma (`TerminalApplication` em `kdeglobals`, lida com `kreadconfig6`/`kreadconfig5`);
4. emuladores conhecidos do ambiente (no KDE, `konsole`);
5. lista genérica de emuladores encontrados no `PATH`.

Configuração:

```toml
[tray]
enabled = true       # mostra o ícone enquanto o daemon estiver ativo
backend = "auto"     # auto | kde | off
terminal = ""        # vazio = detectar automaticamente
```

- `auto` ativa a bandeja apenas quando a sessão é KDE Plasma.
- `kde` força o backend StatusNotifierItem.
- `off` (ou `none`/`noop`) desativa o ícone sem remover as demais preferências.
- O daemon continua funcionando normalmente se a bandeja não estiver disponível; o motivo é registrado em `stderr`.
- O ícone é atualizado a cada verificação periódica do daemon. Sem o daemon em execução, não há ícone.

A arquitetura é abstraída por `TrayBackend`/`TrayHandle`/`TrayManager` em `src/desktop/tray/`. A versão atual implementa apenas o backend KDE StatusNotifierItem; novos ambientes podem ser adicionados implementando o trait e registrando o backend, sem alterar o daemon ou a TUI. A abertura da TUI fica em `src/desktop/terminal.rs` e também é reutilizável por futuros backends.

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
- A inicialização automática suporta apenas o systemd de usuário nesta versão; IPC e rede continuam fora do escopo.
- Outros gerenciadores de serviço podem ser adicionados pelo trait `ServiceBackend`, mas ainda não estão implementados.
- O ícone de bandeja suporta apenas KDE Plasma (StatusNotifierItem) nesta versão. Outros desktops podem ser adicionados pelo trait `TrayBackend`.
- O clique na bandeja abre uma nova instância da TUI; não há IPC para focar uma janela já aberta.
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
- Tasks can repeat daily or on specific weekdays, with an optional time.
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
   - Visual indicators on each day: `•` non-recurring tasks, `◆` events only, `◉` tasks and events, `!` overdue tasks.
   - Recurring tasks are not shown in the calendar; they have their own section organized by weekday.
   - Contextual side panel displaying a detailed list of events and tasks due on the selected date.
   - Quick shortcut `t` to instantly return to today.

3. **General Task List**:
   - Organized table displaying status `[ ]` / `[✓]`, title, priority, due date, user-friendly deadline status, and category/project.
   - Recurring tasks appear with `↻` and their rule (daily or weekly) in the date column.
   - `Space` shortcut to toggle the selected task between completed and incomplete.
   - For recurring tasks, completion applies only to the current day's occurrence (or the date selected in the calendar).
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
   - Recurrence in quick entry:
     - `Academia diariamente 7h` -> repeats every day at 07:00
     - `Revisar seg,qua,sex 09:30` -> repeats Monday, Wednesday, and Friday at 09:30
     - `Feira toda segunda` -> repeats every Monday
   - Completing a recurring task affects only that day's occurrence (or the date selected in the calendar) without disabling the recurrence.
   - Real-time preview of the detected date before confirmation.

8. **Reliable SQLite Persistence**:
   - Normalized tables: `tasks`, `projects`, `events`, `task_completions`, `schema_migrations`.
   - `PRAGMA foreign_keys = ON;` enabled.
   - Automatic incremental migrations (v1 → v4), with safe column, index, and table creation.
   - Data stored at `~/.local/share/calma/calma.db` (or `$XDG_DATA_HOME/calma/calma.db`).

9. **Dedicated Event Calendar**:
   - Dedicated view (`5`) displaying events in chronological order.
   - Quick entry using natural language, with optional date and time (`E`, or `n` while in the event calendar).
   - Editing (`e`) and deletion (`d`) with confirmation.
   - Examples: `Reunião amanhã 14:30-16:00`, `Consulta dia 5 9h`, `Prova 12/11/2026`.
   - Optional end time using a time range: `14:30-16:00` or `14h-16h`.
   - Events also appear in the monthly calendar and in the selected day's side panel.

10. **Recurring Tasks by Weekday**:
   - Dedicated section (`7`) with seven columns: Monday through Sunday.
   - Each recurring task appears in the column of the day it occurs on; daily tasks appear in every column.
   - The current day is visually highlighted and the displayed week is the current week.
   - Navigate columns with `←`/`→` and select tasks with `↑`/`↓`.
   - `Space` completes only that day's occurrence; `e` edits and `d` deletes the series.
   - Completed tasks appear as `[✓]`, and the scheduled time is shown when set.

11. **Persistent Settings**:
   - Dedicated screen (`6`) with **General**, **Notifications**, **Desktop Environment**, and **Appearance** sections.
   - Boolean toggles and validated numeric intervals.
   - Changes are saved immediately to `~/.config/calma/config.toml` (or `$XDG_CONFIG_HOME/calma/config.toml`).
   - Conservative defaults: daemon, automatic startup, and all notification categories are disabled.
   - Invalid values are safely normalized; malformed files are never overwritten automatically.
   - The **Desktop Environment** section is read-only and shows the detected desktop, session type, backend, service availability, and diagnostic message; `r` refreshes detection.

12. **Independent Daemon and Native Notifications**:
   - TUI lifecycle is separate from a daemon process that shares the same SQLite database.
   - Modular service, periodic scheduler, and notification abstraction.
   - Native delivery through the session D-Bus using `org.freedesktop.Notifications`, with verified support on KDE Plasma.
   - One notification per task/event on its day, plus one additional notification within the configured lead time for timed events.
   - Optional automatic startup through a systemd user service created and enabled by the TUI itself, without manually editing service files.
   - KDE system tray icon with three states (good/attention/late vibes) and click-to-open the TUI in the default terminal.
   - Manual foreground execution with `calma daemon`; IPC remains out of scope.

---

## ⌨️ Keyboard Shortcuts

| Key | Action |
|---|---|
| `Tab` | Open the **section menu** (navigate with `↑`/`↓` and confirm with `Enter`) |
| `1` or `m` | Switch to **Monthly Calendar** view |
| `2` or `l` | Switch to **Task List** view |
| `3` or `h` | Switch to **Project Hierarchy** view |
| `4` or `v` | Switch to **Overview (Dashboard)** view |
| `5` | Switch to **Event Calendar** view |
| `6` | Switch to **Settings** screen |
| `7` | Switch to **Recurring Tasks** (columns by weekday) |
| `n` | Open **Quick Entry** for a new task (or a new event in the Event Calendar) |
| `E` | Open **Quick Event Entry** |
| `e` | Edit the selected task, project, or event |
| `Space` | Mark the selected task as completed or reopen it |
| `d` | Delete the selected item (task, project, or event) |
| `t` | Jump to **Today** |
| `x` | **Export** projects, tasks, and events to a JSON file |
| `i` | **Import** data from a JSON file (add or overwrite) |
| `↑` `↓` / `k` `j` | Navigate through lists, the dashboard, or the event calendar |
| `j` / `k` | Select a task in the calendar's side panel |
| `←` `→` | Navigate between weekdays in the calendar |
| `PgUp` / `PgDn` | Previous month / Next month in the calendar |
| `r` | Refresh the desktop environment diagnostics in Settings |
| `?` | Open the help screen with all keyboard shortcuts |
| `Esc` | Close modal / Cancel / Go back |
| `q` | Quit the application |

In the Settings screen: `↑`/`↓` or `j`/`k` navigate, `Enter`/`Space` toggle booleans or start editing numeric values, `r` refreshes environment detection and service availability, `x` exports and `i` imports data, `Esc` returns to the previous view, and `q` quits Calma.

In the task edit form, `Tab`/`Shift+Tab` cycle through Title, Due date, Time, Repeat, Days, Priority, and Project. In Repeat, use `0` (none), `1` (daily), or `2` (weekly); in Days, `1`–`7` toggle Monday through Sunday; in Priority, `1`–`4` choose low, normal, high, and urgent.

---

## 🛠 System Requirements

- **Linux** (Arch Linux, Debian, Ubuntu, Fedora, or any modern distribution).
- **Rust & Cargo** (version 1.80+ recommended).
- SQLite3 library (`libsqlite3-dev` on Debian/Ubuntu, `sqlite` on Arch Linux).
- For desktop notifications: a graphical session with a session D-Bus and an `org.freedesktop.Notifications` service (KDE Plasma is the tested environment).
- For automatic startup: systemd user service (`systemctl --user`). Other service managers are not supported yet.
- For the system tray icon: KDE Plasma with `org.kde.StatusNotifierWatcher` (the icon appears in the background services area) and an installed terminal. Only the KDE backend is implemented in this version.

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

To deliver notifications, the daemon must inherit the graphical session environment, especially `DBUS_SESSION_BUS_ADDRESS` and, when applicable, `WAYLAND_DISPLAY` or `DISPLAY`. The systemd user service is started by the user's own session manager and normally inherits these variables. A service started outside the session (over SSH, in a container, or with an incomplete environment) may not have access to the correct bus. The TUI installs and enables this service automatically; there is no need to create `.service` files manually.

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

#### From within Calma (recommended)

Press `x` on any screen (including Settings) to export projects, tasks, and events to a JSON file. The suggested default path is `~/calma-backup-YYYYMMDD-HHMMSS.json`, and it can be edited before confirming.

The file uses the versioned `calma-backup` format (version `1`) and contains `projects`, `tasks`, `events`, `task_completions`, and `exported_at`. Task recurrence and times are preserved as well. Files generated by older versions without `task_completions` remain valid. Writes are atomic: content goes to a temporary file and is then renamed.

Press `i` to import. Enter the JSON path and choose one of the modes:

- **Import and add**: keeps current data and inserts only items that do not already exist. Comparison ignores IDs, creation timestamps, `sort_order`, and `completed_at`, and normalizes case and whitespace to avoid duplicates. Duplicate projects, tasks, and events are skipped.
- **Import and overwrite**: deletes all current projects, tasks, and events before applying the file. This option is destructive and requires explicit confirmation; the default mode is always add.

Import is transactional and validated before any change: format, version, IDs, cross-references between projects/tasks, and hierarchy cycles. If any item is invalid, nothing in the database is modified.

#### Manual SQLite copy

To back up the complete database:

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
[general]
start_with_system = false
background_monitoring = false
poll_interval_secs = 60

[notifications]
enabled = false
upcoming_tasks = false
overdue_tasks = false
upcoming_events = false
advance_minutes = 60

[notifications.desktop]
backend = "auto"

[tray]
enabled = true
backend = "auto"
terminal = ""
```

Older configurations without `[notifications.desktop]` remain valid and receive `backend = "auto"`. Configurations without `[tray]` receive `enabled = true`, `backend = "auto"`, and `terminal = ""`. Detection results are always runtime-only and are never written to TOML.

### Notification behavior

- Non-completed tasks whose `due_date` or `scheduled_date` is today generate one notification.
- Recurring tasks generate the day-of notification only when their rule matches the current date; a completed occurrence does not notify.
- Tasks due today with a defined time generate a second, distinct notification once they start within `advance_minutes` minutes.
- Overdue tasks generate at most one notification per day.
- Events occurring today generate one notification that day; events that already ended are ignored.
- Timed events generate a second, distinct notification once the event starts within `advance_minutes` minutes.
- All-day events (`start_time` absent) receive only the day-of notification.
- The notification identity includes the kind, item, date, and milestone (day-of or lead time), so the same event can receive both expected notifications without repetition.
- Deduplication is in memory and resets when the day changes. Restarting the daemon may repeat a notification already sent.

### Automatic daemon startup (systemd)

The **Start with the system** option in **Settings > General** handles the entire integration without requiring users to create or edit service files:

- Creates `$XDG_CONFIG_HOME/systemd/user/calma-daemon.service` (or `~/.config/systemd/user/calma-daemon.service`).
- Generates an `ExecStart` pointing to the current Calma executable followed by `daemon`.
- Runs `systemctl --user daemon-reload` and `systemctl --user enable --now calma-daemon.service`.
- Disabling the option runs `systemctl --user disable --now calma-daemon.service`, removes the unit file, and reloads systemd.
- Enabling automatic startup also turns on **background monitoring**, so the daemon has work to do when it starts.
- When the TUI starts with the preference already enabled, the unit is reconciled with the current executable, covering updates and reinstalls.
- If the operation fails, the preference is rolled back and the settings screen shows the reason.

The backend is abstracted by `ServiceBackend`/`AutostartManager` in `src/services/autostart.rs`. The current version implements only the systemd user service; additional managers can be added by implementing the trait and registering the new backend in preference order without changing the TUI.

### System tray icon (KDE)

When the daemon is running in a compatible environment, Calma registers a **StatusNotifierItem** in the KDE tray (the background services area in the bottom-right corner). The icon reflects the effective task state:

| Icon | State | Rule |
|---|---|---|
| good vibes | All caught up | No overdue tasks and no pending tasks for today |
| attention vibes | Tasks for today | At least one unfinished task occurs today |
| late vibes | Overdue tasks | At least one unfinished task has a due date before today |

The rules are the same as the TUI and scheduler: recurring tasks count only when they occur on the date, and completion is checked per occurrence. `late vibes` takes priority over `attention vibes`. The tooltip shows the counts, and the tray menu offers **Abrir Calma** (Open Calma) and **Sair do daemon** (Quit daemon).

Clicking the icon (or **Abrir Calma**) opens the TUI in a separate process using the default terminal detected in this order:

1. `terminal` from the `[tray]` configuration section, when set;
2. the `TERMINAL` environment variable;
3. the KDE Plasma terminal preference (`TerminalApplication` in `kdeglobals`, read through `kreadconfig6`/`kreadconfig5`);
4. known terminals for the detected environment (on KDE, `konsole`);
5. a generic list of terminal emulators found in `PATH`.

Configuration:

```toml
[tray]
enabled = true       # show the icon while the daemon is running
backend = "auto"     # auto | kde | off
terminal = ""        # empty = detect automatically
```

- `auto` enables the tray only when the session is KDE Plasma.
- `kde` forces the StatusNotifierItem backend.
- `off` (or `none`/`noop`) disables the icon without removing other preferences.
- The daemon keeps running normally when the tray is unavailable; the reason is written to `stderr`.
- The icon is refreshed on every periodic daemon check. Without the daemon running, there is no icon.

The architecture is abstracted by `TrayBackend`/`TrayHandle`/`TrayManager` in `src/desktop/tray/`. The current version implements only the KDE StatusNotifierItem backend; new environments can be added by implementing the trait and registering the backend without changing the daemon or the TUI. TUI launching lives in `src/desktop/terminal.rs` and can be reused by future backends.

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
- Automatic startup supports only the systemd user service in this version; IPC and networking remain out of scope.
- Other service managers can be added through the `ServiceBackend` trait but are not implemented yet.
- The tray icon supports only KDE Plasma (StatusNotifierItem) in this version. Other desktops can be added through the `TrayBackend` trait.
- Clicking the tray opens a new TUI instance; there is no IPC to focus an already open window.
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
    │   ├── date_parser.rs
    │   └── portability.rs   # Versioned JSON state export/import
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
