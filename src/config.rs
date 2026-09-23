use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct Config {
    pub resolver: Option<ResolverConfig>,
    pub policy: Option<PolicyConfig>,
    pub output: Option<OutputConfig>,
}

/// Legacy configuration section. Kept for backward compatibility;
/// `preferred_backends` is folded into `policy.prefer` when policy is absent.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ResolverConfig {
    pub preferred_backends: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct PolicyConfig {
    /// Ordered backend preference list. Entries are backend names or the
    /// keywords `native`, `universal`, `container`.
    pub prefer: Option<Vec<String>>,
    /// Backends never chosen automatically (explicit `--backend` still works).
    pub avoid: Option<Vec<String>>,
    pub native: Option<NativePolicy>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NativePolicy {
    /// Prefer the distro's native package manager above universal sources.
    pub prefer_current_distro: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OutputConfig {
    pub color: Option<bool>,
    pub verbose: Option<bool>,
}

/// Well-known preference keywords.
pub const KEYWORD_NATIVE: &str = "native";
pub const KEYWORD_UNIVERSAL: &str = "universal";
pub const KEYWORD_CONTAINER: &str = "container";

/// Default preference order when the user configures nothing.
pub const DEFAULT_PREFER: &[&str] = &[KEYWORD_NATIVE, KEYWORD_UNIVERSAL, KEYWORD_CONTAINER];

/// All valid backend names (kept in sync with `backends::all_backends`).
pub fn known_backend_names() -> Vec<String> {
    crate::backends::backend_names()
}

/// Base configuration directory: `$UNVRS_CONFIG_DIR` if set, otherwise
/// `<user config dir>/unvrs`. Used for `config.toml` and profiles.
pub fn config_dir() -> PathBuf {
    if let Ok(p) = std::env::var("UNVRS_CONFIG_DIR") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unvrs")
}

/// Base data directory: `$UNVRS_DATA_DIR` if set, otherwise
/// `<user data dir>/unvrs`. Used for transaction history.
pub fn data_dir() -> PathBuf {
    if let Ok(p) = std::env::var("UNVRS_DATA_DIR") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unvrs")
}

impl Config {
    /// Load configuration, validating its contents.
    ///
    /// * Missing file            -> defaults
    /// * Unparseable/invalid file -> `ConfigurationError` with the path
    pub fn load() -> crate::error::Result<Self> {
        let path = Self::config_path();
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &Path) -> crate::error::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::UnvrsError::ConfigurationError(format!(
                "cannot read {}: {e}",
                path.display()
            ))
        })?;
        let cfg: Config = toml::from_str(&content).map_err(|e| {
            crate::error::UnvrsError::ConfigurationError(format!(
                "invalid config at {}: {e}",
                path.display()
            ))
        })?;
        cfg.validate(path)?;
        Ok(cfg)
    }

    /// Validate semantics beyond syntax. Returns an error listing every issue.
    pub fn validate(&self, path: &Path) -> crate::error::Result<()> {
        let known = known_backend_names();
        let mut problems = Vec::new();

        let mut entries: Vec<(String, &str)> = Vec::new();
        if let Some(p) = &self.policy {
            for name in p.prefer.iter().flatten() {
                entries.push((name.clone(), "policy.prefer"));
            }
            for name in p.avoid.iter().flatten() {
                entries.push((name.clone(), "policy.avoid"));
            }
        }
        if let Some(r) = &self.resolver {
            for name in r.preferred_backends.iter().flatten() {
                entries.push((name.clone(), "resolver.preferred_backends"));
            }
        }

        for (name, field) in entries {
            let is_keyword =
                [KEYWORD_NATIVE, KEYWORD_UNIVERSAL, KEYWORD_CONTAINER].contains(&name.as_str());
            if !is_keyword && !known.iter().any(|k| k == &name) {
                let hint = crate::suggest::suggest(&name, &known);
                let mut msg = format!("{field}: unknown backend `{name}`");
                if let Some(s) = hint {
                    msg.push_str(&format!(" (did you mean `{s}`?)"));
                }
                problems.push(msg);
            }
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(crate::error::UnvrsError::ConfigurationError(format!(
                "invalid config at {}: {}",
                path.display(),
                problems.join("; ")
            )))
        }
    }

    pub fn config_path() -> PathBuf {
        config_dir().join("config.toml")
    }

    /// Effective preference order (`policy.prefer`, falling back to legacy
    /// `resolver.preferred_backends`, then `DEFAULT_PREFER`).
    pub fn effective_prefer(&self) -> Vec<String> {
        if let Some(p) = &self.policy {
            if let Some(prefer) = &p.prefer {
                if !prefer.is_empty() {
                    return prefer.clone();
                }
            }
        }
        let legacy = self.preferred_backends();
        if !legacy.is_empty() {
            return legacy;
        }
        DEFAULT_PREFER.iter().map(|s| s.to_string()).collect()
    }

    pub fn avoid(&self) -> Vec<String> {
        self.policy
            .as_ref()
            .and_then(|p| p.avoid.clone())
            .unwrap_or_default()
    }

    pub fn prefer_current_distro(&self) -> bool {
        self.policy
            .as_ref()
            .and_then(|p| p.native.as_ref())
            .and_then(|n| n.prefer_current_distro)
            .unwrap_or(true)
    }

    /// Legacy accessor — exact backend names only, keywords stripped.
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
    use std::io::Write;

    #[test]
    fn default_config() {
        let cfg = Config::default();
        assert!(cfg.preferred_backends().is_empty());
        assert!(cfg.color_enabled());
        assert!(!cfg.verbose());
        assert_eq!(
            cfg.effective_prefer(),
            DEFAULT_PREFER
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
        assert!(cfg.avoid().is_empty());
        assert!(cfg.prefer_current_distro());
    }

    #[test]
    fn parse_legacy_config() {
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
        // Legacy list becomes the effective preference order.
        assert_eq!(cfg.effective_prefer(), vec!["pacman", "flatpak"]);
    }

    #[test]
    fn parse_policy_config() {
        let toml = r#"
[policy]
prefer = ["native", "flatpak", "snap"]
avoid = ["snap"]

[policy.native]
prefer_current_distro = false
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.effective_prefer(), vec!["native", "flatpak", "snap"]);
        assert_eq!(cfg.avoid(), vec!["snap"]);
        assert!(!cfg.prefer_current_distro());
    }

    #[test]
    fn policy_prefer_wins_over_legacy_resolver() {
        let toml = r#"
[resolver]
preferred_backends = ["apt"]
[policy]
prefer = ["native"]
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.effective_prefer(), vec!["native"]);
    }

    #[test]
    fn invalid_toml_is_an_error_with_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "not [valid toml").unwrap();
        let err = Config::load_from_path(&path).unwrap_err();
        assert!(err.to_string().contains("invalid config"));
    }

    #[test]
    fn unknown_backend_in_policy_is_rejected_with_suggestion() {
        let cfg = Config {
            policy: Some(PolicyConfig {
                prefer: Some(vec!["fltapak".into()]),
                avoid: None,
                native: None,
            }),
            ..Default::default()
        };
        let err = cfg.validate(Path::new("test.toml")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("fltapak"));
        assert!(msg.contains("flatpak"), "expected suggestion in: {msg}");
    }

    #[test]
    fn keywords_are_valid_policy_entries() {
        let cfg = Config {
            policy: Some(PolicyConfig {
                prefer: Some(vec![
                    "native".into(),
                    "universal".into(),
                    "container".into(),
                ]),
                avoid: Some(vec!["snap".into()]),
                native: None,
            }),
            ..Default::default()
        };
        assert!(cfg.validate(Path::new("t.toml")).is_ok());
    }

    #[test]
    fn missing_file_yields_defaults() {
        let cfg = Config::load_from_path(Path::new("/nonexistent/unvrs-cfg.toml")).unwrap();
        assert_eq!(cfg, Config::default());
    }
}
