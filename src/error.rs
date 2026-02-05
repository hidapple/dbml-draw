use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to parse DBML: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Layout file error: {0}")]
    LayoutError(String),
}
