use crate::error::{Result, UnvrsError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A named list of packages applied through the normal resolver/plan pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub name: String,
    pub packages: Vec<String>,
    /// Optional per-profile default backend.
    #[serde(default)]
    pub backend: Option<String>,
}

pub fn profile_dir() -> PathBuf {
    crate::config::config_dir().join("profiles")
}

fn profile_path(name: &str) -> PathBuf {
    profile_dir().join(format!("{name}.toml"))
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(UnvrsError::ConfigurationError(format!(
            "invalid profile name `{name}` (use letters, digits, - and _ only)"
        )));
    }
    Ok(())
}

pub fn create(name: &str, packages: Vec<String>) -> Result<Profile> {
    validate_name(name)?;
    let path = profile_path(name);
    if path.exists() {
        return Err(UnvrsError::ConfigurationError(format!(
            "profile `{name}` already exists at {}",
            path.display()
        )));
    }
    let profile = Profile {
        name: name.to_string(),
        packages,
        backend: None,
    };
    save(&profile)?;
    Ok(profile)
}

pub fn save(profile: &Profile) -> Result<()> {
    validate_name(&profile.name)?;
    let path = profile_path(&profile.name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(profile)
        .map_err(|e| UnvrsError::ConfigurationError(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn load(name: &str) -> Result<Profile> {
    validate_name(name)?;
    let path = profile_path(name);
    if !path.exists() {
        let available = list_names();
        let hint = crate::suggest::suggest(name, &available);
        let mut msg = format!("profile not found: {name}");
        if let Some(s) = hint {
            msg.push_str(&format!(" (did you mean `{s}`?)"));
        }
        return Err(UnvrsError::ConfigurationError(msg));
    }
    let content = std::fs::read_to_string(&path)?;
    let profile: Profile = toml::from_str(&content).map_err(|e| {
        UnvrsError::ConfigurationError(format!("invalid profile {}: {e}", path.display()))
    })?;
    if profile.packages.is_empty() {
        return Err(UnvrsError::ConfigurationError(format!(
            "profile `{name}` has no packages"
        )));
    }
    Ok(profile)
}

pub fn list() -> Result<Vec<Profile>> {
    let dir = profile_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(p) = load(stem) {
                    profiles.push(p);
                }
            }
        }
    }
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

pub fn list_names() -> Vec<String> {
    list()
        .map(|ps| ps.iter().map(|p| p.name.clone()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_profile_names_rejected() {
        assert!(validate_name("").is_err());
        assert!(validate_name("../evil").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name("ok-name_1").is_ok());
    }

    #[test]
    fn profile_toml_roundtrip() {
        let p = Profile {
            name: "dev".into(),
            packages: vec!["git".into(), "neovim".into()],
            backend: None,
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let back: Profile = toml::from_str(&s).unwrap();
        assert_eq!(p, back);
    }
}
