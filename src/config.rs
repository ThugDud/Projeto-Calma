use std::fs;
use std::path::PathBuf;

/// Configuração do aplicativo Calma.
#[derive(Debug, Clone)]
pub struct Config {
    /// Caminho para o banco de dados SQLite.
    pub database_path: PathBuf,
    /// Tema visual (escuro por padrão).
    pub theme: String,
    /// Notificações habilitadas.
    pub notifications_enabled: bool,
    /// Atalho para ir para hoje.
    pub goto_today_key: char,
    /// Atalho para sair.
    pub quit_key: char,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = data_home_dir().unwrap_or_else(|| PathBuf::from("."));
        let calma_dir = data_dir.join("calma");
        let _ = fs::create_dir_all(&calma_dir);

        Config {
            database_path: calma_dir.join("calma.db"),
            theme: "dark".to_string(),
            notifications_enabled: false,
            goto_today_key: 't',
            quit_key: 'q',
        }
    }
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

impl Config {
    /// Cria uma nova instância com valores padrão.
    pub fn new() -> Self {
        Self::default()
    }
}
