//! Detecção do terminal padrão e abertura da TUI em um processo separado.
//!
//! A detecção é separada da execução para permanecer testável e para
//! permitir novos critérios no futuro. A ordem de preferência é:
//!
//! 1. `tray.terminal` na configuração (quando preenchido);
//! 2. variável de ambiente `TERMINAL`;
//! 3. preferência do KDE Plasma (`TerminalApplication` em `kdeglobals`);
//! 4. emuladores conhecidos do ambiente detectado;
//! 5. lista genérica de emuladores encontrados no `PATH`.
//!
//! A execução é destacada do daemon: o terminal é iniciado com a TUI como
//! filho e o processo é colhido em uma thread própria para não deixar
//! zumbis enquanto o daemon permanece vivo.

use crate::config::Config;
use crate::desktop::detection::{DesktopEnvironment, DesktopSession};
use crate::errors::{AppError, AppResult};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

/// Programa e argumentos usados para abrir a TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSpec {
    /// Executável do terminal.
    pub program: String,
    /// Argumentos que precedem o executável do Calma.
    pub args: Vec<String>,
}

impl TerminalSpec {
    /// Cria uma especificação a partir do nome do terminal.
    ///
    /// Os argumentos são derivados do emulador conhecido (por exemplo,
    /// `konsole -e` e `gnome-terminal --`).
    pub fn new(program: impl Into<String>) -> Self {
        let program = program.into();
        let args = terminal_args(&program);
        Self { program, args }
    }

    /// Cria uma especificação preservando argumentos explícitos.
    pub fn with_args(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }

    /// Monta o programa e a lista final de argumentos para abrir `executable`.
    pub fn command(&self, executable: &Path) -> (PathBuf, Vec<String>) {
        let mut args = self.args.clone();
        args.push(executable.to_string_lossy().into_owned());
        (PathBuf::from(&self.program), args)
    }
}

/// Lê `$TERMINAL` e a preferência do KDE para detectar o terminal padrão.
pub fn detect(override_value: &str, session: &DesktopSession) -> Option<TerminalSpec> {
    let terminal_var = std::env::var("TERMINAL").ok();
    let kde_terminal = if session.environment.is_kde() {
        kde_terminal_from_config()
    } else {
        None
    };

    select_terminal(
        override_value,
        terminal_var.as_deref(),
        kde_terminal.as_deref(),
        &session.environment,
        command_exists,
    )
}

/// Versão pura da detecção, usada pelos testes.
///
/// `is_available` decide se um programa pode ser usado. O valor explícito
/// de `override_value` sempre vence e não passa por essa verificação, para
/// que uma configuração do usuário produza um erro claro ao abrir em vez de
/// ser silenciosamente ignorada.
pub fn select_terminal(
    override_value: &str,
    terminal_var: Option<&str>,
    kde_terminal: Option<&str>,
    environment: &DesktopEnvironment,
    is_available: impl Fn(&str) -> bool,
) -> Option<TerminalSpec> {
    let configured = override_value.trim();
    if !configured.is_empty() {
        return Some(parse_terminal_value(configured));
    }

    if let Some(value) = terminal_var
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let spec = parse_terminal_value(value);
        if is_available(&spec.program) {
            return Some(spec);
        }
    }

    if let Some(value) = kde_terminal
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let spec = parse_terminal_value(value);
        if is_available(&spec.program) {
            return Some(spec);
        }
    }

    for program in preferred_terminals(environment) {
        if is_available(program) {
            return Some(TerminalSpec::new(*program));
        }
    }

    for program in GENERIC_TERMINALS {
        if is_available(program) {
            return Some(TerminalSpec::new(*program));
        }
    }

    None
}

const GENERIC_TERMINALS: &[&str] = &[
    "x-terminal-emulator",
    "konsole",
    "gnome-terminal",
    "kgx",
    "ptyxis",
    "kitty",
    "alacritty",
    "wezterm",
    "foot",
    "xfce4-terminal",
    "mate-terminal",
    "qterminal",
    "tilix",
    "terminator",
    "xterm",
];

fn preferred_terminals(environment: &DesktopEnvironment) -> &'static [&'static str] {
    match environment {
        DesktopEnvironment::KdePlasma => &["konsole"],
        DesktopEnvironment::Gnome => &["kgx", "gnome-terminal", "ptyxis"],
        DesktopEnvironment::Xfce => &["xfce4-terminal"],
        DesktopEnvironment::Cinnamon => &["gnome-terminal", "x-terminal-emulator"],
        DesktopEnvironment::Mate => &["mate-terminal"],
        DesktopEnvironment::Lxqt => &["qterminal"],
        DesktopEnvironment::I3 | DesktopEnvironment::Sway => {
            &["alacritty", "kitty", "foot", "wezterm", "xterm"]
        }
        DesktopEnvironment::Unknown(_) => &[],
        DesktopEnvironment::Headless => &[],
    }
}

/// Separa um valor como `kitty --single-instance` em programa e argumentos.
fn parse_terminal_value(value: &str) -> TerminalSpec {
    let mut parts = value.split_whitespace();
    let program = parts.next().unwrap_or_default().to_string();
    let mut args: Vec<String> = parts.map(|part| part.to_string()).collect();
    args.extend(terminal_args(&program));
    TerminalSpec { program, args }
}

/// Argumentos que fazem o emulador executar o comando informado.
fn terminal_args(program: &str) -> Vec<String> {
    let name = Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();

    let args: &[&str] = match name.as_str() {
        // Emuladores que aceitam o comando diretamente.
        "kitty" | "foot" => &[],
        // O GNOME Terminal separa opções do comando com `--`.
        "gnome-terminal" | "kgx" | "ptyxis" => &["--"],
        // WezTerm exige o subcomando `start`.
        "wezterm" => &["start", "--"],
        "xfce4-terminal" | "terminator" => &["-x"],
        // Compatíveis com xterm e a maioria dos demais.
        _ => &["-e"],
    };

    args.iter().map(|arg| (*arg).to_string()).collect()
}

/// Abre a TUI no terminal padrão em um processo independente.
pub trait TuiLauncher: fmt::Debug + Send + Sync {
    fn launch(&self) -> AppResult<()>;
}

/// Implementação baseada em [`TerminalSpec`].
#[derive(Debug, Clone)]
pub struct TerminalLauncher {
    executable: PathBuf,
    spec: Option<TerminalSpec>,
}

impl TerminalLauncher {
    /// Detecta o terminal e usa o executável atual do Calma.
    pub fn for_current_exe(config: &Config, session: &DesktopSession) -> AppResult<Self> {
        let executable = std::env::current_exe().map_err(|err| {
            AppError::Tray(format!(
                "não foi possível descobrir o executável do Calma: {}",
                err
            ))
        })?;

        Ok(Self {
            executable,
            spec: detect(&config.tray.terminal, session),
        })
    }

    /// Cria um lançador explícito, útil para testes e integrações futuras.
    pub fn with_spec(executable: PathBuf, spec: Option<TerminalSpec>) -> Self {
        Self { executable, spec }
    }

    pub fn spec(&self) -> Option<&TerminalSpec> {
        self.spec.as_ref()
    }
}

impl TuiLauncher for TerminalLauncher {
    fn launch(&self) -> AppResult<()> {
        let spec = self.spec.as_ref().ok_or_else(|| {
            AppError::Tray(
                "nenhum terminal padrão encontrado; defina `tray.terminal` na configuração"
                    .to_string(),
            )
        })?;

        let (program, args) = spec.command(&self.executable);
        let mut child = Command::new(&program)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                AppError::Tray(format!(
                    "não foi possível abrir a TUI com {}: {}",
                    program.display(),
                    err
                ))
            })?;

        // O daemon é de longa duração; colher o filho evita zumbis quando o
        // usuário fecha a janela do terminal.
        thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(())
    }
}

fn kde_terminal_from_config() -> Option<String> {
    for tool in ["kreadconfig6", "kreadconfig5"] {
        if !command_exists(tool) {
            continue;
        }

        let output = Command::new(tool)
            .args([
                "--file",
                "kdeglobals",
                "--group",
                "General",
                "--key",
                "TerminalApplication",
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }

    None
}

fn command_exists(program: &str) -> bool {
    if program.trim().is_empty() {
        return false;
    }

    let path = Path::new(program);
    if path.components().count() > 1 {
        return path.is_file();
    }

    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                let candidate = dir.join(program);
                candidate.is_file()
            })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn only(allowed: &'static [&'static str]) -> impl Fn(&str) -> bool {
        move |program| allowed.contains(&program)
    }

    #[test]
    fn explicit_configuration_wins_even_when_not_detected() {
        let spec = select_terminal(
            "meu-terminal",
            Some("alacritty"),
            Some("konsole"),
            &DesktopEnvironment::KdePlasma,
            only(&["alacritty", "konsole"]),
        )
        .expect("terminal configurado");

        assert_eq!(spec.program, "meu-terminal");
        assert_eq!(spec.args, vec!["-e"]);
    }

    #[test]
    fn env_terminal_takes_precedence_over_kde_preference() {
        let spec = select_terminal(
            "",
            Some("alacritty"),
            Some("konsole"),
            &DesktopEnvironment::KdePlasma,
            only(&["alacritty", "konsole"]),
        )
        .expect("terminal");

        assert_eq!(spec.program, "alacritty");
    }

    #[test]
    fn kde_preference_is_used_before_the_builtin_list() {
        let spec = select_terminal(
            "",
            None,
            Some("kitty"),
            &DesktopEnvironment::KdePlasma,
            only(&["kitty", "konsole"]),
        )
        .expect("terminal");

        assert_eq!(spec.program, "kitty");
    }

    #[test]
    fn falls_back_to_konsole_on_kde() {
        let spec = select_terminal(
            "",
            None,
            None,
            &DesktopEnvironment::KdePlasma,
            only(&["konsole"]),
        )
        .expect("terminal");

        assert_eq!(spec.program, "konsole");
        assert_eq!(spec.args, vec!["-e"]);
    }

    #[test]
    fn no_terminal_available_returns_none() {
        assert!(
            select_terminal("", None, None, &DesktopEnvironment::Headless, |_| false,).is_none()
        );
    }

    #[test]
    fn terminal_argument_conventions_are_respected() {
        let cases = [
            ("konsole", vec!["-e"]),
            ("gnome-terminal", vec!["--"]),
            ("wezterm", vec!["start", "--"]),
            ("kitty", vec![]),
            ("foot", vec![]),
            ("xfce4-terminal", vec!["-x"]),
            ("xterm", vec!["-e"]),
        ];

        for (program, expected) in cases {
            assert_eq!(
                TerminalSpec::new(program).args,
                expected,
                "programa: {program}"
            );
        }
    }

    #[test]
    fn terminal_value_with_extra_arguments_is_preserved() {
        let spec = parse_terminal_value("kitty --single-instance");
        assert_eq!(spec.program, "kitty");
        assert_eq!(spec.args, vec!["--single-instance"]);
    }

    #[test]
    fn command_appends_the_calma_executable() {
        let spec = TerminalSpec::new("konsole");
        let (program, args) = spec.command(Path::new("/usr/bin/calma"));

        assert_eq!(program, PathBuf::from("konsole"));
        assert_eq!(args, vec!["-e", "/usr/bin/calma"]);
    }

    #[test]
    fn launcher_without_terminal_reports_a_clear_error() {
        let launcher = TerminalLauncher::with_spec(PathBuf::from("/usr/bin/calma"), None);
        let error = launcher.launch().expect_err("sem terminal");
        assert!(error.to_string().contains("tray.terminal"));
    }

    #[test]
    fn launcher_spawns_the_configured_program() {
        let launcher = TerminalLauncher::with_spec(
            PathBuf::from("/usr/bin/calma"),
            Some(TerminalSpec::with_args("/bin/true", Vec::new())),
        );

        launcher.launch().expect("processo deve ser iniciado");
    }
}
