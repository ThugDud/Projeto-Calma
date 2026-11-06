use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Nome do diretório do aplicativo dentro dos diretórios XDG.
pub const APP_DIR_NAME: &str = "calma";
/// Nome do arquivo de configuração persistente.
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Intervalo mínimo aceito para verificações do daemon (5 segundos).
pub const MIN_POLL_INTERVAL_SECS: u64 = 5;
/// Intervalo máximo aceito para verificações do daemon (1 hora).
pub const MAX_POLL_INTERVAL_SECS: u64 = 60 * 60;
/// Intervalo padrão de verificação: 60 segundos.
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;

/// Antecedência mínima de notificações: 1 minuto.
pub const MIN_NOTIFICATION_ADVANCE_MINUTES: u32 = 1;
/// Antecedência máxima de notificações: 7 dias.
pub const MAX_NOTIFICATION_ADVANCE_MINUTES: u32 = 7 * 24 * 60;
/// Antecedência padrão de notificações: 30 minutos.
pub const DEFAULT_NOTIFICATION_ADVANCE_MINUTES: u32 = 30;

/// Preferências gerais do Calma.
///
/// Os padrões são conservadores: nenhum recurso opcional é ativado
/// automaticamente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    /// Iniciar o daemon automaticamente ao entrar no sistema.
    ///
    /// Nesta fase a preferência é apenas armazenada. A integração com
    /// systemd ainda não está implementada.
    pub start_with_system: bool,
    /// Habilitar o monitoramento periódico em segundo plano pelo daemon.
    pub background_monitoring: bool,
    /// Intervalo entre verificações do daemon, em segundos.
    pub poll_interval_secs: u64,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            start_with_system: false,
            background_monitoring: false,
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
        }
    }
}

/// Preferências do subsistema de notificações.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationSettings {
    /// Habilita notificações da área de trabalho.
    ///
    /// A entrega real ainda não está implementada; esta opção controla
    /// apenas a geração/dispatch de notificações pelo daemon.
    pub enabled: bool,
    /// Notificar sobre prazos de tarefas que estão se aproximando.
    pub upcoming_tasks: bool,
    /// Notificar sobre tarefas atrasadas.
    pub overdue_tasks: bool,
    /// Notificar sobre eventos que estão se aproximando.
    pub upcoming_events: bool,
    /// Quantos minutos antes do prazo/evento a notificação deve ser gerada.
    pub advance_minutes: u32,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            upcoming_tasks: false,
            overdue_tasks: false,
            upcoming_events: false,
            advance_minutes: DEFAULT_NOTIFICATION_ADVANCE_MINUTES,
        }
    }
}

/// Preferências visuais da interface.
///
/// O tema ainda não altera a renderização atual; o campo já existe para
/// que a preferência seja persistida quando o seletor de temas for criado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InterfaceSettings {
    /// Nome do tema visual desejado.
    pub theme: String,
}

impl Default for InterfaceSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
        }
    }
}

/// Configuração centralizada do aplicativo Calma.
///
/// A configuração é persistida em `$XDG_CONFIG_HOME/calma/config.toml`
/// (ou `~/.config/calma/config.toml`). O banco de dados continua sendo
/// armazenado em `$XDG_DATA_HOME/calma/calma.db`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Caminho do arquivo de configuração. Não é serializado.
    #[serde(skip)]
    pub config_path: PathBuf,
    /// Caminho do banco SQLite compartilhado entre TUI e daemon.
    #[serde(skip)]
    pub database_path: PathBuf,
    /// Aviso produzido durante o carregamento, útil para exibir na TUI.
    #[serde(skip)]
    pub load_warning: Option<String>,
    /// Preferências gerais.
    pub general: GeneralSettings,
    /// Preferências de notificações.
    pub notifications: NotificationSettings,
    /// Preferências visuais.
    pub interface: InterfaceSettings,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = data_home_dir().unwrap_or_else(|| PathBuf::from("."));
        let calma_data_dir = data_dir.join(APP_DIR_NAME);
        // Mantém o comportamento anterior: garante que o diretório de dados
        // exista antes de o SQLite ser aberto.
        let _ = fs::create_dir_all(&calma_data_dir);

        Self {
            config_path: default_config_path(),
            database_path: calma_data_dir.join("calma.db"),
            load_warning: None,
            general: GeneralSettings::default(),
            notifications: NotificationSettings::default(),
            interface: InterfaceSettings::default(),
        }
    }
}

impl Config {
    /// Cria uma configuração em memória com os valores padrão.
    ///
    /// Para carregar a configuração persistente do usuário, use
    /// [`Config::load`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Carrega a configuração do caminho XDG padrão.
    ///
    /// Arquivos ausentes são criados com os valores padrão. Arquivos
    /// inválidos nunca são sobrescritos automaticamente: o Calma usa os
    /// padrões em memória e registra um aviso em `load_warning`.
    pub fn load() -> Self {
        Self::load_from(&default_config_path())
    }

    /// Carrega a configuração de um caminho específico.
    ///
    /// Esta função é usada pelos testes e permite que o daemon e a TUI
    /// compartilhem exatamente a mesma configuração.
    pub fn load_from(path: &Path) -> Self {
        let defaults = Config::default();

        let mut config = if path.exists() {
            match fs::read_to_string(path) {
                Ok(contents) => match toml::from_str::<Config>(&contents) {
                    Ok(config) => config,
                    Err(err) => Config {
                        load_warning: Some(format!(
                            "Não foi possível interpretar {}: {}. Usando valores padrão.",
                            path.display(),
                            err
                        )),
                        ..Config::default()
                    },
                },
                Err(err) => Config {
                    load_warning: Some(format!(
                        "Não foi possível ler {}: {}. Usando valores padrão.",
                        path.display(),
                        err
                    )),
                    ..Config::default()
                },
            }
        } else {
            let mut created = Config::default();
            if let Err(err) = created.save_to(path) {
                created.load_warning = Some(format!(
                    "Não foi possível criar a configuração padrão em {}: {}",
                    path.display(),
                    err
                ));
            }
            created
        };

        config.config_path = path.to_path_buf();
        config.database_path = defaults.database_path;

        let warnings = config.validate();
        if !warnings.is_empty() {
            let validation_warning = warnings.join(" ");
            config.load_warning = Some(match config.load_warning.take() {
                Some(existing) => format!("{} {}", existing, validation_warning),
                None => validation_warning,
            });
        }

        config
    }

    /// Valida e normaliza valores carregados externamente.
    ///
    /// Valores numéricos fora dos limites são ajustados para a faixa
    /// aceita, e uma mensagem descritiva é retornada para cada ajuste.
    pub fn validate(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.general.poll_interval_secs < MIN_POLL_INTERVAL_SECS {
            warnings.push(format!(
                "poll_interval_secs abaixo do mínimo ({}); ajustado para {}.",
                MIN_POLL_INTERVAL_SECS, MIN_POLL_INTERVAL_SECS
            ));
            self.general.poll_interval_secs = MIN_POLL_INTERVAL_SECS;
        } else if self.general.poll_interval_secs > MAX_POLL_INTERVAL_SECS {
            warnings.push(format!(
                "poll_interval_secs acima do máximo ({}); ajustado para {}.",
                MAX_POLL_INTERVAL_SECS, MAX_POLL_INTERVAL_SECS
            ));
            self.general.poll_interval_secs = MAX_POLL_INTERVAL_SECS;
        }

        if self.notifications.advance_minutes < MIN_NOTIFICATION_ADVANCE_MINUTES {
            warnings.push(format!(
                "advance_minutes abaixo do mínimo ({}); ajustado para {}.",
                MIN_NOTIFICATION_ADVANCE_MINUTES, MIN_NOTIFICATION_ADVANCE_MINUTES
            ));
            self.notifications.advance_minutes = MIN_NOTIFICATION_ADVANCE_MINUTES;
        } else if self.notifications.advance_minutes > MAX_NOTIFICATION_ADVANCE_MINUTES {
            warnings.push(format!(
                "advance_minutes acima do máximo ({}); ajustado para {}.",
                MAX_NOTIFICATION_ADVANCE_MINUTES, MAX_NOTIFICATION_ADVANCE_MINUTES
            ));
            self.notifications.advance_minutes = MAX_NOTIFICATION_ADVANCE_MINUTES;
        }

        if self.interface.theme.trim().is_empty() {
            warnings.push("O tema não pode ficar vazio; usando \"dark\".".to_string());
            self.interface.theme = InterfaceSettings::default().theme;
        }

        warnings
    }

    /// Persiste a configuração no caminho associado a esta instância.
    pub fn save(&self) -> AppResult<()> {
        self.save_to(&self.config_path)
    }

    /// Persiste a configuração no caminho informado.
    ///
    /// A escrita é feita em um arquivo temporário e depois renomeada, para
    /// reduzir o risco de deixar uma configuração truncada em disco.
    pub fn save_to(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                AppError::Config(format!(
                    "Não foi possível criar o diretório {}: {}",
                    parent.display(),
                    err
                ))
            })?;
        }

        let serialized = toml::to_string_pretty(self)
            .map_err(|err| AppError::Config(format!("Erro ao serializar configuração: {}", err)))?;

        let temp_path = path.with_extension("toml.tmp");
        fs::write(&temp_path, serialized).map_err(|err| {
            AppError::Config(format!(
                "Não foi possível escrever {}: {}",
                temp_path.display(),
                err
            ))
        })?;

        fs::rename(&temp_path, path).map_err(|err| {
            let _ = fs::remove_file(&temp_path);
            AppError::Config(format!(
                "Não foi possível salvar {}: {}",
                path.display(),
                err
            ))
        })?;

        Ok(())
    }
}

/// Retorna o caminho padrão do arquivo de configuração.
pub fn default_config_path() -> PathBuf {
    config_home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR_NAME)
        .join(CONFIG_FILE_NAME)
}

/// Retorna o diretório de configuração do usuário seguindo XDG_CONFIG_HOME,
/// com fallback para ~/.config quando não definida.
fn config_home_dir() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("XDG_CONFIG_HOME") {
        if !val.is_empty() {
            return Some(PathBuf::from(val));
        }
    }
    home_dir().map(|mut p| {
        p.push(".config");
        p
    })
}

/// Retorna o diretório de dados do usuário seguindo XDG_DATA_HOME,
/// com fallback para ~/.local/share quando não definida.
fn data_home_dir() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("XDG_DATA_HOME") {
        if !val.is_empty() {
            return Some(PathBuf::from(val));
        }
    }
    home_dir().map(|mut p| {
        p.push(".local");
        p.push("share");
        p
    })
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("relógio do sistema")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "calma-config-test-{}-{}-{}",
                std::process::id(),
                nanos,
                name
            ))
            .join(CONFIG_FILE_NAME)
    }

    fn cleanup(path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn defaults_are_conservative() {
        let config = Config::default();

        assert!(!config.general.start_with_system);
        assert!(!config.general.background_monitoring);
        assert!(!config.notifications.enabled);
        assert!(!config.notifications.upcoming_tasks);
        assert!(!config.notifications.overdue_tasks);
        assert!(!config.notifications.upcoming_events);
        assert_eq!(
            config.general.poll_interval_secs,
            DEFAULT_POLL_INTERVAL_SECS
        );
        assert_eq!(
            config.notifications.advance_minutes,
            DEFAULT_NOTIFICATION_ADVANCE_MINUTES
        );
        assert_eq!(config.interface.theme, "dark");
    }

    #[test]
    fn missing_file_creates_defaults_without_user_intervention() {
        let path = temp_config_path("missing");
        assert!(!path.exists());

        let config = Config::load_from(&path);

        assert!(path.exists(), "o arquivo padrão deve ser criado");
        assert!(config.load_warning.is_none());
        assert_eq!(config.config_path, path);
        assert!(!config.general.background_monitoring);

        let reloaded = Config::load_from(&path);
        assert_eq!(reloaded.general, config.general);
        assert_eq!(reloaded.notifications, config.notifications);

        cleanup(&path);
    }

    #[test]
    fn save_and_load_roundtrip_preserves_settings() {
        let path = temp_config_path("roundtrip");
        let config = Config {
            config_path: path.clone(),
            general: GeneralSettings {
                start_with_system: true,
                background_monitoring: true,
                poll_interval_secs: 120,
            },
            notifications: NotificationSettings {
                enabled: true,
                upcoming_events: false,
                advance_minutes: 45,
                ..NotificationSettings::default()
            },
            interface: InterfaceSettings {
                theme: "claro".to_string(),
            },
            ..Config::default()
        };

        config.save_to(&path).expect("configuração deve ser salva");
        let loaded = Config::load_from(&path);

        assert_eq!(loaded.general, config.general);
        assert_eq!(loaded.notifications, config.notifications);
        assert_eq!(loaded.interface, config.interface);
        assert!(loaded.load_warning.is_none());

        cleanup(&path);
    }

    #[test]
    fn invalid_numeric_values_are_clamped_on_load() {
        let path = temp_config_path("invalid-numbers");
        let contents = r#"
[general]
poll_interval_secs = 0

[notifications]
advance_minutes = 999999
"#;
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();

        let config = Config::load_from(&path);

        assert_eq!(config.general.poll_interval_secs, MIN_POLL_INTERVAL_SECS);
        assert_eq!(
            config.notifications.advance_minutes,
            MAX_NOTIFICATION_ADVANCE_MINUTES
        );
        assert!(config.load_warning.is_some());

        cleanup(&path);
    }

    #[test]
    fn malformed_file_uses_defaults_and_is_not_overwritten() {
        let path = temp_config_path("malformed");
        let original = "isto não é TOML válido = [";
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, original).unwrap();

        let config = Config::load_from(&path);

        assert!(config.load_warning.is_some());
        assert_eq!(fs::read_to_string(&path).unwrap(), original);

        cleanup(&path);
    }

    #[test]
    fn missing_nested_fields_fall_back_to_defaults() {
        let path = temp_config_path("partial");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "[general]\nstart_with_system = true\n").unwrap();

        let config = Config::load_from(&path);

        assert!(config.general.start_with_system);
        assert_eq!(
            config.general.poll_interval_secs,
            DEFAULT_POLL_INTERVAL_SECS
        );
        assert_eq!(
            config.notifications.advance_minutes,
            DEFAULT_NOTIFICATION_ADVANCE_MINUTES
        );

        cleanup(&path);
    }
}
