use thiserror::Error;

#[derive(Debug, Error)]
pub enum OviaError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type OviaResult<T> = Result<T, OviaError>;
