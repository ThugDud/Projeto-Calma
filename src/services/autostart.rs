//! Inicialização automática do daemon em serviços do sistema.
//!
//! O gerenciamento é abstraído por [`ServiceBackend`] para permitir novos
//! backends no futuro (por exemplo, OpenRC, launchd ou um serviço de usuário
//! específico de um ambiente gráfico). Nesta versão apenas o systemd de
//! usuário é suportado.
//!
//! A TUI cria, habilita e desabilita o serviço automaticamente, sem exigir
//! que o usuário escreva arquivos `.service` manualmente.

use crate::errors::{AppError, AppResult};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

/// Nome do serviço gerenciado pelo Calma.
pub const SERVICE_NAME: &str = "calma-daemon.service";

/// Implementações de gerenciamento de serviço disponíveis.
///
/// Novos backends devem adicionar uma variante aqui, implementar
/// [`ServiceBackend`] e ser registrados em [`AutostartManager`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceBackendKind {
    /// Serviço de usuário gerenciado por `systemctl --user`.
    SystemdUser,
}

impl ServiceBackendKind {
    pub fn label(self) -> &'static str {
        match self {
            ServiceBackendKind::SystemdUser => "systemd (usuário)",
        }
    }
}

impl fmt::Display for ServiceBackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Especificação do processo que o serviço deve iniciar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSpec {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
}

impl ServiceSpec {
    /// Usa o executável atual do Calma para iniciar `daemon`.
    pub fn current_daemon() -> AppResult<Self> {
        let executable = std::env::current_exe().map_err(|err| {
            AppError::Service(format!(
                "não foi possível descobrir o caminho do executável do Calma: {}",
                err
            ))
        })?;

        Ok(Self {
            executable,
            arguments: vec!["daemon".to_string()],
        })
    }

    fn exec_start(&self) -> String {
        let mut parts = vec![quote_systemd_token(&self.executable.to_string_lossy())];
        parts.extend(
            self.arguments
                .iter()
                .map(|argument| quote_systemd_token(argument)),
        );
        parts.join(" ")
    }
}

/// Executa comandos externos de forma injetável para testes.
pub trait CommandRunner: fmt::Debug + Send + Sync {
    fn run(&self, program: &Path, args: &[String]) -> AppResult<()>;
}

/// Executor real que inicia processos do sistema.
#[derive(Debug, Default)]
pub struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn run(&self, program: &Path, args: &[String]) -> AppResult<()> {
        let output = Command::new(program).args(args).output().map_err(|err| {
            AppError::Service(format!(
                "não foi possível executar {}: {}",
                program.display(),
                err
            ))
        })?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        let detail = if detail.is_empty() {
            format!("código de saída {:?}", output.status.code())
        } else {
            detail
        };

        Err(AppError::Service(format!(
            "o comando {} {} falhou: {}",
            program.display(),
            args.join(" "),
            detail
        )))
    }
}

/// Backend de gerenciamento de serviço.
pub trait ServiceBackend: fmt::Debug + Send + Sync {
    fn kind(&self) -> ServiceBackendKind;

    /// Indica se o gerenciador parece disponível no sistema.
    fn is_available(&self) -> bool;

    /// Instala/atualiza o serviço e o habilita para iniciar com o usuário.
    fn enable(&self, spec: &ServiceSpec) -> AppResult<()>;

    /// Desabilita e remove o serviço.
    fn disable(&self) -> AppResult<()>;
}

/// Gerencia o serviço de usuário do systemd.
///
/// O arquivo é criado em `$XDG_CONFIG_HOME/systemd/user/calma-daemon.service`
/// (ou `~/.config/systemd/user/calma-daemon.service`).
#[derive(Debug)]
pub struct SystemdUserService {
    unit_name: String,
    unit_dir: PathBuf,
    systemctl: PathBuf,
    runner: Arc<dyn CommandRunner>,
}

impl SystemdUserService {
    pub fn new(unit_dir: PathBuf, systemctl: PathBuf, runner: Arc<dyn CommandRunner>) -> Self {
        Self {
            unit_name: SERVICE_NAME.to_string(),
            unit_dir,
            systemctl,
            runner,
        }
    }

    /// Caminho completo do arquivo de unidade gerenciado.
    pub fn unit_path(&self) -> PathBuf {
        self.unit_dir.join(&self.unit_name)
    }

    /// Conteúdo do arquivo `.service` gerado.
    pub fn unit_contents(&self, spec: &ServiceSpec) -> String {
        format!(
            "# Gerado automaticamente pelo Calma. Não edite este arquivo manualmente.\n\
             [Unit]\n\
             Description=Calma - daemon de tarefas e eventos\n\
             \n\
             [Service]\n\
             Type=simple\n\
             ExecStart={}\n\
             Restart=on-failure\n\
             RestartSec=5\n\
             \n\
             [Install]\n\
             WantedBy=default.target\n",
            spec.exec_start()
        )
    }

    fn run_systemctl(&self, args: &[&str]) -> AppResult<()> {
        let mut full_args = Vec::with_capacity(args.len() + 1);
        full_args.push("--user".to_string());
        full_args.extend(args.iter().map(|arg| arg.to_string()));
        self.runner.run(&self.systemctl, &full_args)
    }

    fn write_unit(&self, spec: &ServiceSpec) -> AppResult<PathBuf> {
        fs::create_dir_all(&self.unit_dir).map_err(|err| {
            AppError::Service(format!(
                "não foi possível criar {}: {}",
                self.unit_dir.display(),
                err
            ))
        })?;

        let path = self.unit_path();
        let temp = path.with_extension("service.tmp");
        fs::write(&temp, self.unit_contents(spec)).map_err(|err| {
            AppError::Service(format!(
                "não foi possível escrever {}: {}",
                temp.display(),
                err
            ))
        })?;
        fs::rename(&temp, &path).map_err(|err| {
            let _ = fs::remove_file(&temp);
            AppError::Service(format!(
                "não foi possível salvar {}: {}",
                path.display(),
                err
            ))
        })?;

        Ok(path)
    }
}

impl Default for SystemdUserService {
    fn default() -> Self {
        Self::new(
            systemd_user_unit_dir(),
            PathBuf::from("systemctl"),
            Arc::new(ProcessCommandRunner),
        )
    }
}

impl ServiceBackend for SystemdUserService {
    fn kind(&self) -> ServiceBackendKind {
        ServiceBackendKind::SystemdUser
    }

    fn is_available(&self) -> bool {
        command_exists(&self.systemctl)
    }

    fn enable(&self, spec: &ServiceSpec) -> AppResult<()> {
        let unit_path = self.write_unit(spec)?;

        let result = self
            .run_systemctl(&["daemon-reload"])
            .and_then(|_| self.run_systemctl(&["enable", "--now", &self.unit_name]));

        if let Err(err) = result {
            // Não deixa uma unidade órfã se o systemd não estiver disponível.
            let _ = fs::remove_file(&unit_path);
            let _ = self.run_systemctl(&["daemon-reload"]);
            return Err(err);
        }

        Ok(())
    }

    fn disable(&self) -> AppResult<()> {
        let unit_path = self.unit_path();
        if !unit_path.exists() {
            return Ok(());
        }

        self.run_systemctl(&["disable", "--now", &self.unit_name])?;

        fs::remove_file(&unit_path).map_err(|err| {
            AppError::Service(format!(
                "não foi possível remover {}: {}",
                unit_path.display(),
                err
            ))
        })?;

        // A recarga apenas atualiza o estado do systemd; a remoção do arquivo
        // já garante que a unidade não seja habilitada novamente.
        let _ = self.run_systemctl(&["daemon-reload"]);
        Ok(())
    }
}

/// Seleciona e aciona o primeiro backend de serviço disponível.
///
/// A ordem dos backends define a preferência. Com apenas um backend nesta
/// versão, o systemd de usuário é usado quando `systemctl` está presente.
#[derive(Debug, Clone)]
pub struct AutostartManager {
    backends: Vec<Arc<dyn ServiceBackend>>,
}

impl Default for AutostartManager {
    fn default() -> Self {
        Self::systemd()
    }
}

impl AutostartManager {
    /// Gerencia apenas o systemd de usuário (backend inicial).
    pub fn systemd() -> Self {
        Self::with_backends(vec![Arc::new(SystemdUserService::default())])
    }

    /// Cria um gerenciador com backends explícitos, na ordem de preferência.
    ///
    /// Útil para testes e para registrar futuros backends sem alterar a TUI.
    pub fn with_backends(backends: Vec<Arc<dyn ServiceBackend>>) -> Self {
        Self { backends }
    }

    /// Primeiro backend disponível, se houver.
    pub fn available_backend(&self) -> Option<ServiceBackendKind> {
        self.backends
            .iter()
            .find(|backend| backend.is_available())
            .map(|backend| backend.kind())
    }

    /// Instala e habilita o serviço no primeiro backend disponível.
    pub fn enable(&self, spec: &ServiceSpec) -> AppResult<ServiceBackendKind> {
        let mut attempts = Vec::new();

        for backend in &self.backends {
            if !backend.is_available() {
                attempts.push(format!("{} indisponível", backend.kind().label()));
                continue;
            }

            match backend.enable(spec) {
                Ok(()) => return Ok(backend.kind()),
                Err(err) => attempts.push(format!("{}: {}", backend.kind().label(), err)),
            }
        }

        Err(AppError::Service(format!(
            "nenhum gerenciador de serviços compatível conseguiu habilitar o daemon. {}",
            if attempts.is_empty() {
                "Nenhum backend configurado.".to_string()
            } else {
                attempts.join(" | ")
            }
        )))
    }

    /// Desabilita e remove o serviço no primeiro backend disponível.
    pub fn disable(&self) -> AppResult<ServiceBackendKind> {
        let mut attempts = Vec::new();

        for backend in &self.backends {
            if !backend.is_available() {
                attempts.push(format!("{} indisponível", backend.kind().label()));
                continue;
            }

            match backend.disable() {
                Ok(()) => return Ok(backend.kind()),
                Err(err) => attempts.push(format!("{}: {}", backend.kind().label(), err)),
            }
        }

        Err(AppError::Service(format!(
            "nenhum gerenciador de serviços compatível conseguiu desabilitar o daemon. {}",
            if attempts.is_empty() {
                "Nenhum backend configurado.".to_string()
            } else {
                attempts.join(" | ")
            }
        )))
    }
}

/// Garante que o serviço reflita a preferência salva.
///
/// Chamado na inicialização da TUI para instalar/atualizar a unidade caso a
/// configuração já esteja habilitada (por exemplo, após atualizar o Calma).
pub fn reconcile_start_with_system(
    config: &crate::config::Config,
) -> AppResult<Option<ServiceBackendKind>> {
    if !config.general.start_with_system {
        return Ok(None);
    }

    let manager = AutostartManager::default();
    let spec = ServiceSpec::current_daemon()?;
    manager.enable(&spec).map(Some)
}

fn systemd_user_unit_dir() -> PathBuf {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));

    config_home.join("systemd").join("user")
}

fn command_exists(program: &Path) -> bool {
    if program.components().count() > 1 {
        return program.is_file();
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

/// Escapa um token para a diretiva `ExecStart` do systemd.
fn quote_systemd_token(value: &str) -> String {
    let escaped = value.replace('%', "%%");
    if escaped.is_empty() {
        return "\"\"".to_string();
    }

    if escaped
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '\\' | '$' | ';'))
    {
        format!("\"{}\"", escaped.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        escaped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Debug, Default)]
    struct RecordingRunner {
        calls: Mutex<Vec<Vec<String>>>,
    }

    impl RecordingRunner {
        fn calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().expect("calls").clone()
        }
    }

    impl CommandRunner for RecordingRunner {
        fn run(&self, program: &Path, args: &[String]) -> AppResult<()> {
            let mut call = vec![program.display().to_string()];
            call.extend(args.iter().cloned());
            self.calls.lock().expect("calls").push(call);
            Ok(())
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("relógio")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "calma-autostart-test-{}-{}-{}",
            std::process::id(),
            nanos,
            name
        ))
    }

    fn spec() -> ServiceSpec {
        ServiceSpec {
            executable: PathBuf::from("/usr/bin/calma"),
            arguments: vec!["daemon".to_string()],
        }
    }

    #[test]
    fn unit_file_starts_daemon_and_is_enabled_for_default_target() {
        let dir = temp_dir("unit");
        let service = SystemdUserService::new(
            dir.clone(),
            PathBuf::from("systemctl"),
            Arc::new(RecordingRunner::default()),
        );

        let contents = service.unit_contents(&spec());
        assert!(contents.contains("ExecStart=/usr/bin/calma daemon"));
        assert!(contents.contains("WantedBy=default.target"));
        assert!(contents.contains("Restart=on-failure"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn enable_writes_unit_and_calls_systemctl() {
        let dir = temp_dir("enable");
        let runner = Arc::new(RecordingRunner::default());
        let service =
            SystemdUserService::new(dir.clone(), PathBuf::from("systemctl"), runner.clone());

        service.enable(&spec()).expect("habilitar serviço");

        let unit_path = dir.join(SERVICE_NAME);
        assert!(unit_path.exists());
        let contents = fs::read_to_string(&unit_path).expect("ler unidade");
        assert!(contents.contains("ExecStart=/usr/bin/calma daemon"));

        let calls = runner.calls();
        assert_eq!(calls[0], vec!["systemctl", "--user", "daemon-reload"]);
        assert_eq!(
            calls[1],
            vec![
                "systemctl",
                "--user",
                "enable",
                "--now",
                "calma-daemon.service"
            ]
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn disable_stops_service_and_removes_unit() {
        let dir = temp_dir("disable");
        let runner = Arc::new(RecordingRunner::default());
        let service =
            SystemdUserService::new(dir.clone(), PathBuf::from("systemctl"), runner.clone());
        service.enable(&spec()).expect("criar unidade");

        service.disable().expect("desabilitar serviço");

        assert!(!dir.join(SERVICE_NAME).exists());
        let calls = runner.calls();
        assert!(calls.iter().any(|call| {
            call == &vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "disable".to_string(),
                "--now".to_string(),
                "calma-daemon.service".to_string(),
            ]
        }));

        let _ = fs::remove_dir_all(dir);
    }

    #[derive(Debug, Default)]
    struct FailingRunner;

    impl CommandRunner for FailingRunner {
        fn run(&self, _program: &Path, _args: &[String]) -> AppResult<()> {
            Err(AppError::Service("falha simulada".into()))
        }
    }

    #[test]
    fn failed_enable_removes_the_orphan_unit() {
        let dir = temp_dir("enable-failure");
        let service = SystemdUserService::new(
            dir.clone(),
            PathBuf::from("systemctl"),
            Arc::new(FailingRunner),
        );

        let error = service.enable(&spec()).expect_err("deve falhar");
        assert!(error.to_string().contains("falha simulada"));
        assert!(!dir.join(SERVICE_NAME).exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn quoted_executable_with_spaces_is_escaped() {
        let service = SystemdUserService::new(
            PathBuf::from("/tmp"),
            PathBuf::from("systemctl"),
            Arc::new(RecordingRunner::default()),
        );
        let spec = ServiceSpec {
            executable: PathBuf::from("/home/user/Meus Programas/calma"),
            arguments: vec!["daemon".to_string()],
        };

        assert!(
            service
                .unit_contents(&spec)
                .contains("ExecStart=\"/home/user/Meus Programas/calma\" daemon")
        );
    }
}
