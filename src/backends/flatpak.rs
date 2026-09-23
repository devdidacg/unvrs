use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct FlatpakBackend;

impl Default for FlatpakBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FlatpakBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for FlatpakBackend {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn is_available(&self) -> bool {
        executor::is_available("flatpak")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux
    }

    fn is_universal(&self) -> bool {
        true
    }

    fn requires_root(&self) -> bool {
        // System-wide installs need privileges; flatpak also supports
        // --user installs. Treat as root-required for safety messaging.
        true
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("flatpak", ["--version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("flatpak", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("flatpak search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 3 {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let description = parts.get(1).map(|s| s.trim().to_string());
                let app_id = parts.get(2).map(|s| s.trim().to_string());
                Some(PackageCandidate {
                    name: app_id.unwrap_or(name),
                    version: None,
                    source: PackageSource::Flatpak,
                    backend: "flatpak".into(),
                    architecture: None,
                    description,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("flatpak", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut homepage = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Name: ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version: ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Summary: ") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Homepage: ") {
                homepage = Some(val.to_string());
            }
        }

        if name.is_empty() {
            name = package.to_string();
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::Flatpak,
            backend: "flatpak".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "flatpak",
            ["install", "-y", "flathub", package],
        ))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "flatpak",
            ["uninstall", "-y", package],
        ))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "flatpak",
            ["update", "--appstream"],
        ))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("flatpak", ["update", "-y"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("flatpak", &["list", "--columns=application,version"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "flatpak list".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if parts.is_empty() {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = parts.get(1).unwrap_or(&"").trim().to_string();
                if name.is_empty() {
                    return None;
                }
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::Flatpak,
                    backend: "flatpak".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
