use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Animated spinner. Disabled entirely in JSON mode (so machine-readable
/// stdout stays clean) and when stdout is not a TTY.
///
/// If dropped without an explicit `stop_with`/`stop_fail` (e.g. an error
/// returned via `?` mid-command), it silently stops so the terminal line is
/// cleared before the error message prints.
pub struct Spinner {
    spinning: Arc<AtomicBool>,
    active: AtomicBool,
}

impl Spinner {
    pub fn new(msg: &str) -> Self {
        if crate::context::json() || !is_tty() {
            return Self {
                spinning: Arc::new(AtomicBool::new(false)),
                active: AtomicBool::new(false),
            };
        }

        let spinning = Arc::new(AtomicBool::new(true));
        let spinning_clone = spinning.clone();
        let msg = msg.to_string();
        let no_color = crate::context::no_color();

        thread::spawn(move || {
            let frames = ["-", "\\", "|", "/"];
            let mut i = 0;
            while spinning_clone.load(Ordering::Relaxed) {
                let line = if no_color {
                    format!("\r  {}...", msg)
                } else {
                    format!("\r  \x1b[36m{}\x1b[0m {}...", frames[i % frames.len()], msg)
                };
                print!("{}", line);
                io::stdout().flush().ok();
                i += 1;
                thread::sleep(Duration::from_millis(100));
            }
            if !no_color {
                print!("\r\x1b[K");
            } else {
                print!("\r");
            }
            io::stdout().flush().ok();
        });

        Self {
            spinning,
            active: AtomicBool::new(true),
        }
    }

    /// Claim ownership of stopping; returns true the first time only.
    fn claim(&self) -> bool {
        self.active.swap(false, Ordering::Relaxed)
    }

    pub fn stop_with(&self, msg: &str) {
        if !self.claim() {
            return;
        }
        self.spinning.store(false, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(130));
        if crate::context::no_color() {
            println!("  OK {}", msg);
        } else {
            println!("  \x1b[32m✓\x1b[0m {}", msg);
        }
    }

    pub fn stop_fail(&self, msg: &str) {
        if !self.claim() {
            return;
        }
        self.spinning.store(false, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(130));
        if crate::context::no_color() {
            println!("  FAIL {}", msg);
        } else {
            println!("  \x1b[31m✗\x1b[0m {}", msg);
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if self.claim() {
            self.spinning.store(false, Ordering::Relaxed);
            // Give the animation thread time to clear the line before the
            // caller prints anything else (error messages, etc.).
            thread::sleep(Duration::from_millis(130));
        }
    }
}

fn is_tty() -> bool {
    use std::io::IsTerminal;
    io::stdout().is_terminal()
}

pub fn icon_ok() -> &'static str {
    if crate::context::no_color() {
        "OK"
    } else {
        "\x1b[32m✓\x1b[0m"
    }
}

pub fn icon_fail() -> &'static str {
    if crate::context::no_color() {
        "FAIL"
    } else {
        "\x1b[31m✗\x1b[0m"
    }
}

pub fn icon_warn() -> &'static str {
    if crate::context::no_color() {
        "WARN"
    } else {
        "\x1b[33m!\x1b[0m"
    }
}

pub fn icon_info() -> &'static str {
    if crate::context::no_color() {
        "*"
    } else {
        "\x1b[36m*\x1b[0m"
    }
}

pub fn dim(text: &str) -> String {
    if crate::context::no_color() {
        text.to_string()
    } else {
        format!("\x1b[2m{text}\x1b[0m")
    }
}

pub fn bold(text: &str) -> String {
    if crate::context::no_color() {
        text.to_string()
    } else {
        format!("\x1b[1m{text}\x1b[0m")
    }
}

pub fn green(text: &str) -> String {
    if crate::context::no_color() {
        text.to_string()
    } else {
        format!("\x1b[32m{text}\x1b[0m")
    }
}

pub fn red(text: &str) -> String {
    if crate::context::no_color() {
        text.to_string()
    } else {
        format!("\x1b[31m{text}\x1b[0m")
    }
}

pub fn cyan(text: &str) -> String {
    if crate::context::no_color() {
        text.to_string()
    } else {
        format!("\x1b[36m{text}\x1b[0m")
    }
}

pub fn yellow(text: &str) -> String {
    if crate::context::no_color() {
        text.to_string()
    } else {
        format!("\x1b[33m{text}\x1b[0m")
    }
}
