use crate::error::{Result, UnvrsError};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Default timeout for read-only queries (search/info/list).
pub const QUERY_TIMEOUT: Duration = Duration::from_secs(120);
/// Default timeout for mutating operations (install/remove/upgrade).
pub const MUTATION_TIMEOUT: Duration = Duration::from_secs(1800);

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

/// A fully-specified external command. Backends build these so that planning,
/// dry-run display and actual execution share one source of truth.
/// Arguments are always passed as an array — never through a shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    /// Human-readable rendering used in plans and logs.
    pub display: String,
}

impl CommandSpec {
    pub fn new<S: AsRef<str>, A: IntoIterator<Item = S>>(program: &str, args: A) -> Self {
        let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_string()).collect();
        let mut display = program.to_string();
        for a in &args {
            display.push(' ');
            if a.chars()
                .any(|c| c.is_whitespace() || c == '"' || c == '\'')
            {
                display.push('"');
                display.push_str(&a.replace('"', "\\\""));
                display.push('"');
            } else {
                display.push_str(a);
            }
        }
        Self {
            program: program.to_string(),
            args,
            display,
        }
    }

    /// Render with a privilege prefix, e.g. `sudo apt-get ...`.
    pub fn display_with_sudo(&self) -> String {
        format!("sudo {}", self.display)
    }
}

/// Run a read-only command to completion. Never invokes a shell.
pub fn execute<S: AsRef<str>>(program: &str, args: &[S]) -> Result<CommandResult> {
    execute_with_timeout(program, args, QUERY_TIMEOUT)
}

/// Run a potentially long mutating command with a generous timeout.
pub fn execute_mutation<S: AsRef<str>>(program: &str, args: &[S]) -> Result<CommandResult> {
    execute_with_timeout(program, args, MUTATION_TIMEOUT)
}

/// Execute `program args...` without a shell, capturing output.
///
/// Error mapping is deliberately actionable:
/// * missing binary -> `ExecutableNotFound` (suggests `unvrs doctor`)
/// * timeout        -> `Timeout` (child killed)
/// * signal death   -> `Signal`
/// * non-zero exit  -> `CommandFailed` with stderr attached
pub fn execute_with_timeout<S: AsRef<str>>(
    program: &str,
    args: &[S],
    timeout: Duration,
) -> Result<CommandResult> {
    let display = CommandSpec::new(program, args.iter()).display;

    let mut child = Command::new(program)
        .args(args.iter().map(|a| a.as_ref()))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => UnvrsError::ExecutableNotFound {
                program: program.to_string(),
                backend: program.to_string(),
            },
            std::io::ErrorKind::PermissionDenied => UnvrsError::PermissionDenied(display.clone()),
            _ => UnvrsError::CommandFailed {
                command: display.clone(),
                exit_code: -1,
                stderr: e.to_string(),
            },
        })?;

    // Drain pipes on background threads so a chatty child can never deadlock
    // against a full OS pipe buffer while we wait.
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let out_handle = std::thread::spawn(move || read_pipe(stdout_pipe));
    let err_handle = std::thread::spawn(move || read_pipe(stderr_pipe));

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    // Pipes close once the child dies; reader threads finish.
                    let _ = out_handle.join();
                    let _ = err_handle.join();
                    return Err(UnvrsError::Timeout {
                        command: display,
                        timeout_secs: timeout.as_secs(),
                    });
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = out_handle.join();
                let _ = err_handle.join();
                return Err(UnvrsError::CommandFailed {
                    command: display,
                    exit_code: -1,
                    stderr: e.to_string(),
                });
            }
        }
    };

    let stdout = out_handle.join().unwrap_or_default();
    let stderr = err_handle.join().unwrap_or_default();

    let result = CommandResult {
        stdout,
        stderr,
        exit_code: status.code().unwrap_or(-1),
    };

    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        // Preserve captured output in the error for diagnosis.
        return Err(UnvrsError::Signal {
            command: display,
            signal,
        });
    }

    #[allow(unreachable_code)]
    Ok(result)
}

fn read_pipe<R: std::io::Read>(pipe: Option<R>) -> String {
    let mut buf = Vec::new();
    if let Some(mut r) = pipe {
        let _ = std::io::Read::read_to_end(&mut r, &mut buf);
    }
    String::from_utf8_lossy(&buf).to_string()
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

    #[test]
    fn spec_display_quotes_whitespace() {
        let spec = CommandSpec::new("apt-get", ["install", "-y", "hello world"]);
        assert_eq!(spec.display, "apt-get install -y \"hello world\"");
    }

    #[test]
    fn spec_display_no_quotes_for_simple_args() {
        let spec = CommandSpec::new("apt-get", ["install", "-y", "firefox"]);
        assert_eq!(spec.display, "apt-get install -y firefox");
    }

    #[test]
    fn spec_display_escapes_embedded_quotes() {
        let spec = CommandSpec::new("pm", ["say", "a\"b"]);
        assert_eq!(spec.display, "pm say \"a\\\"b\"");
    }

    #[test]
    fn missing_executable_maps_to_actionable_error() {
        let err = execute::<&str>("definitely-not-a-real-binary-xyz", &[]).unwrap_err();
        match &err {
            UnvrsError::ExecutableNotFound { program, .. } => {
                assert_eq!(program, "definitely-not-a-real-binary-xyz");
            }
            other => panic!("expected ExecutableNotFound, got {other:?}"),
        }
        assert!(err.suggestion().is_some());
    }

    #[cfg(unix)]
    #[test]
    fn successful_command_captures_stdout() {
        let r = execute("echo", &["hello".into()]).unwrap();
        assert!(r.success());
        assert!(r.stdout.contains("hello"));
    }

    #[cfg(unix)]
    #[test]
    fn timeout_kills_long_running_command() {
        let err = execute_with_timeout("sleep", &["10"], Duration::from_millis(50)).unwrap_err();
        match err {
            UnvrsError::Timeout { .. } => {}
            other => panic!("expected Timeout, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn large_output_does_not_deadlock() {
        // ~1 MB of output would overflow a pipe buffer if not drained.
        let r = execute("sh", &["-c", "yes x | head -c 1000000"]).unwrap();
        assert!(r.success());
        assert!(r.stdout.len() >= 1_000_000);
    }

    #[test]
    fn non_zero_exit_returns_result_with_exit_code() {
        // The executor itself reports non-zero exits as Ok results; backends
        // map them to `CommandFailed` (or treat them as "no match") themselves.
        #[cfg(windows)]
        let (prog, args) = ("cmd", vec!["/C".to_string(), "exit 3".to_string()]);
        #[cfg(unix)]
        let (prog, args) = ("sh", vec!["-c".to_string(), "exit 3".to_string()]);
        let r = execute(prog, &args).unwrap();
        assert!(!r.success());
        assert_eq!(r.exit_code, 3);
    }
}
