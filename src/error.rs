use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnvrsError {
    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("package not found: {0}")]
    PackageNotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("command failed: {command} (exit code: {exit_code})\nstderr: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("configuration error: {0}")]
    ConfigurationError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, UnvrsError>;
