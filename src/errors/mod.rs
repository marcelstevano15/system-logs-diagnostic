use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Journal error: {0}")]
    Journal(String),

    #[error("Search engine error: {0}")]
    Search(#[from] tantivy::TantivyError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Export error: {0}")]
    Export(String),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Watch error: {0}")]
    Watch(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("State error: {0}")]
    State(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;

