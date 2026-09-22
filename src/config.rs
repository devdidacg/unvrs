use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    pub resolver: Option<ResolverConfig>,
    pub output: Option<OutputConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResolverConfig {
    pub preferred_backends: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputConfig {
    pub color: Option<bool>,
    pub verbose: Option<bool>,
}

impl Config {
    pub fn load() -> Self {
        Self::load_from_path(&Self::config_path())
    }

    pub fn load_from_path(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("unvrs")
            .join("config.toml")
    }

    pub fn preferred_backends(&self) -> Vec<String> {
        self.resolver
            .as_ref()
            .and_then(|r| r.preferred_backends.clone())
            .unwrap_or_default()
    }

    pub fn color_enabled(&self) -> bool {
        self.output.as_ref().and_then(|o| o.color).unwrap_or(true)
    }

    pub fn verbose(&self) -> bool {
        self.output
            .as_ref()
            .and_then(|o| o.verbose)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = Config::default();
        assert!(cfg.preferred_backends().is_empty());
        assert!(cfg.color_enabled());
        assert!(!cfg.verbose());
    }

    #[test]
    fn parse_config() {
        let toml = r#"
[resolver]
preferred_backends = ["pacman", "flatpak"]

[output]
color = false
verbose = true
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.preferred_backends(), vec!["pacman", "flatpak"]);
        assert!(!cfg.color_enabled());
        assert!(cfg.verbose());
    }
}
