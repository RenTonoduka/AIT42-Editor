use thiserror::Error;

/// Session repository error types
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Alias for Result with SessionError
pub type Result<T> = std::result::Result<T, SessionError>;

impl From<SessionError> for String {
    fn from(err: SessionError) -> Self {
        err.to_string()
    }
}
