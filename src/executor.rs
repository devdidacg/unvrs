use crate::error::{Result, UnvrsError};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CommandResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

pub fn execute(program: &str, args: &[&str]) -> Result<CommandResult> {
    let output =
        Command::new(program)
            .args(args)
            .output()
            .map_err(|e| UnvrsError::CommandFailed {
                command: format!("{program} {}", args.join(" ")),
                exit_code: -1,
                stderr: e.to_string(),
            })?;

    Ok(CommandResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

pub fn execute_sudo(program: &str, args: &[&str]) -> Result<CommandResult> {
    let output = Command::new("sudo")
        .arg(program)
        .args(args)
        .output()
        .map_err(|e| UnvrsError::CommandFailed {
            command: format!("sudo {program} {}", args.join(" ")),
            exit_code: -1,
            stderr: e.to_string(),
        })?;

    Ok(CommandResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

pub fn is_available(program: &str) -> bool {
    which::which(program).is_ok()
}

pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::getuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

pub fn is_sudo_available() -> bool {
    is_available("sudo")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_result_success() {
        let r = CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        };
        assert!(r.success());
    }

    #[test]
    fn command_result_failure() {
        let r = CommandResult {
            stdout: String::new(),
            stderr: "error".into(),
            exit_code: 1,
        };
        assert!(!r.success());
    }
}
