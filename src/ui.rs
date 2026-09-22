use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Spinner {
    spinning: Arc<AtomicBool>,
}

impl Spinner {
    pub fn new(msg: &str) -> Self {
        let spinning = Arc::new(AtomicBool::new(true));
        let spinning_clone = spinning.clone();
        let msg = msg.to_string();
        let msg_clone = msg.clone();

        thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut i = 0;
            while spinning_clone.load(Ordering::Relaxed) {
                print!("\r  {} {} ", frames[i % frames.len()], msg_clone);
                io::stdout().flush().ok();
                i += 1;
                thread::sleep(Duration::from_millis(80));
            }
            print!("\r\x1b[K");
            io::stdout().flush().ok();
        });

        Self { spinning }
    }

    pub fn stop_with(&self, msg: &str) {
        self.spinning.store(false, Ordering::Relaxed);
        // Give the thread a moment to exit
        thread::sleep(Duration::from_millis(100));
        println!("  {}", msg);
    }
}

pub fn icon_ok() -> &'static str {
    "\x1b[32m✓\x1b[0m"
}

pub fn icon_fail() -> &'static str {
    "\x1b[31m✗\x1b[0m"
}

pub fn icon_warn() -> &'static str {
    "\x1b[33m⚠\x1b[0m"
}

pub fn icon_info() -> &'static str {
    "\x1b[36m●\x1b[0m"
}

pub fn dim(text: &str) -> String {
    format!("\x1b[2m{text}\x1b[0m")
}

pub fn bold(text: &str) -> String {
    format!("\x1b[1m{text}\x1b[0m")
}
