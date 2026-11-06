/// Erros principais do aplicativo Calma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    /// Erro de entrada/saída do terminal.
    Terminal(String),
    /// Erro de banco de dados.
    Database(String),
    /// Erro de entrada do usuário (dados inválidos, campos ausentes, etc.).
    Input(String),
    /// Erro de regra de negócio (ex.: ciclo na hierarquia, tarefa não encontrada).
    Business(String),
    /// Erro de serialização/desserialização.
    Serialization(String),
    /// Erro ao carregar, validar ou salvar configurações.
    Config(String),
    /// Operação não implementada ainda.
    NotImplemented(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Terminal(msg) => write!(f, "Erro de terminal: {}", msg),
            AppError::Database(msg) => write!(f, "Erro de banco de dados: {}", msg),
            AppError::Input(msg) => write!(f, "Entrada inválida: {}", msg),
            AppError::Business(msg) => write!(f, "Erro de regra de negócio: {}", msg),
            AppError::Serialization(msg) => write!(f, "Erro de serialização: {}", msg),
            AppError::Config(msg) => write!(f, "Erro de configuração: {}", msg),
            AppError::NotImplemented(msg) => write!(f, "Não implementado: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Terminal(err.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::Input(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
