use std::sync::OnceLock;

/// Global output/process context, initialized once from CLI flags before any
/// output happens. Uses `OnceLock` instead of `static mut` so concurrent
/// reads from the spinner thread are safe.
#[derive(Debug, Clone, Copy)]
pub struct OutputContext {
    pub no_color: bool,
    pub json: bool,
    pub verbosity: u8,
}

static CTX: OnceLock<OutputContext> = OnceLock::new();

pub fn init(no_color: bool, json: bool, verbosity: u8) {
    let _ = CTX.set(OutputContext {
        no_color,
        json,
        verbosity,
    });
}

fn get() -> OutputContext {
    *CTX.get().unwrap_or(&OutputContext {
        no_color: false,
        json: false,
        verbosity: 0,
    })
}

pub fn no_color() -> bool {
    get().no_color
}

pub fn json() -> bool {
    get().json
}

pub fn verbosity() -> u8 {
    get().verbosity
}

pub fn log_verbose(level: u8, msg: &str) {
    if verbosity() >= level && !json() {
        eprintln!("  {}", crate::ui::dim(msg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_before_init() {
        // Not initialized in this test process: getters must not panic.
        let _ = verbosity();
        let _ = json();
    }
}
