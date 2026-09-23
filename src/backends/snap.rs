use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct SnapBackend;

impl Default for SnapBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for SnapBackend {
    fn name(&self) -> &'static str {
        "snap"
    }

    fn is_available(&self) -> bool {
        executor::is_available("snap")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux
    }

    fn is_universal(&self) -> bool {
        true
    }

    fn requires_root(&self) -> bool {
        true
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("snap", ["version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("snap", &["find", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("snap find {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 3 {
                    return None;
                }
                let name = parts[0].to_string();
                let version = Some(parts[1].to_string());
                let desc = parts[2..].join(" ");
                Some(PackageCandidate {
                    name,
                    version,
                    source: PackageSource::Snap,
                    backend: "snap".into(),
                    architecture: None,
                    description: if desc.is_empty() { None } else { Some(desc) },
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("snap", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut homepage = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("name: ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("latest/") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("summary: ") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("website: ") {
                homepage = Some(val.to_string());
            }
        }

        if name.is_empty() {
            name = package.to_string();
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::Snap,
            backend: "snap".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("snap", ["install", package]))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("snap", ["remove", package]))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("snap", ["refresh"]))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("snap", ["refresh"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("snap", &["list"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "snap list".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    return None;
                }
                let name = parts[0].to_string();
                let version = parts[1].to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::Snap,
                    backend: "snap".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
