use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnvrsError {
    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("unknown backend: {name}")]
    UnknownBackend {
        name: String,
        /// Closest known backend name, if any (for "did you mean" hints).
        suggestion: Option<String>,
    },

    #[error("package not found: {0}")]
    PackageNotFound(String),

    #[error("invalid package name: {0}")]
    InvalidPackageName(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("`{program}` was not found on this system")]
    ExecutableNotFound { program: String, backend: String },

    #[error("command timed out after {timeout_secs}s: {command}")]
    Timeout { command: String, timeout_secs: u64 },

    #[error(
        "command failed: {command} (exit code: {exit_code}){}",
        stderr_snippet(stderr)
    )]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("command terminated by signal {signal}: {command}")]
    Signal { command: String, signal: i32 },

    #[error("unsafe operation requires confirmation: {0}")]
    RequiresCrossDistro(String),

    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("configuration error: {0}")]
    ConfigurationError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl UnvrsError {
    /// A concrete, actionable suggestion shown to the user on failure.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            UnvrsError::ExecutableNotFound { .. } => {
                Some("Run `unvrs doctor` to inspect available backends.".into())
            }
            UnvrsError::BackendUnavailable(_) => {
                Some("Run `unvrs doctor` to inspect available backends.".into())
            }
            UnvrsError::UnknownBackend { .. } => {
                Some("Run `unvrs doctor` to see valid backend names.".into())
            }
            UnvrsError::RequiresCrossDistro(_) => {
                Some("Re-run with `--cross-distro` if you really want a non-native backend.".into())
            }
            UnvrsError::InvalidPackageName(_) => {
                Some("Package names may contain letters, digits and @ . _ + - / : = only.".into())
            }
            UnvrsError::PermissionDenied(_) => {
                Some("Re-run with sufficient privileges, e.g. `sudo unvrs ...`.".into())
            }
            UnvrsError::PackageNotFound(_) => {
                Some("Try `unvrs search <name>` to see where the package exists.".into())
            }
            UnvrsError::ConfigurationError(msg) => {
                if msg.contains("profile not found") {
                    Some("Run `unvrs profile list` to see existing profiles.".into())
                } else if msg.contains("no transaction")
                    || msg.contains("invalid profile name")
                    || msg.contains("already exists")
                {
                    // The message itself already carries the right hint.
                    None
                } else {
                    Some("Fix the file shown above or run `unvrs doctor`.".into())
                }
            }
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, UnvrsError>;

/// Render a short, indented stderr snippet for error display (max ~400 chars).
fn stderr_snippet(stderr: &str) -> String {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let count = trimmed.chars().count();
    let snippet: String = trimmed.chars().take(400).collect();
    let suffix = if count > 400 { "…" } else { "" };
    format!("\n  {snippet}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions_exist_for_actionable_errors() {
        let e = UnvrsError::ExecutableNotFound {
            program: "apt".into(),
            backend: "apt".into(),
        };
        assert!(e.suggestion().is_some());

        let e = UnvrsError::UnknownBackend {
            name: "fltapak".into(),
            suggestion: Some("flatpak".into()),
        };
        assert!(e.suggestion().is_some());
    }

    #[test]
    fn error_display_does_not_panic_with_quotes() {
        let e = UnvrsError::PackageNotFound("foo\"bar".into());
        assert!(e.to_string().contains("foo\"bar"));
    }
}
