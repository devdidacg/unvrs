use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct BrewBackend;

impl Default for BrewBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl BrewBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for BrewBackend {
    fn name(&self) -> &'static str {
        "brew"
    }

    fn is_available(&self) -> bool {
        executor::is_available("brew")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::MacOS || os.family == OsFamily::Linux
    }

    fn is_universal(&self) -> bool {
        true
    }

    fn requires_root(&self) -> bool {
        false
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("brew", ["--version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("brew", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("brew search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| PackageCandidate {
                name: line.trim().to_string(),
                version: None,
                source: PackageSource::System,
                backend: "brew".into(),
                architecture: None,
                description: None,
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("brew", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let homepage = None;

        for line in result.stdout.lines().take(10) {
            if line.starts_with('/') || line.contains(':') && line.contains(' ') {
                // First line usually: name: version
                if let Some(pos) = line.find(':') {
                    let n = line[..pos].trim();
                    let v = line[pos + 1..].trim();
                    if !n.is_empty() {
                        name = n.to_string();
                    }
                    if !v.is_empty() {
                        version = Some(v.to_string());
                    }
                }
            }
            if let Some(val) = line.strip_prefix("=> ") {
                if description.is_none() {
                    description = Some(val.trim().to_string());
                }
            }
        }

        if name.is_empty() {
            name = package.to_string();
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "brew".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("brew", ["install", package]))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("brew", ["uninstall", package]))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("brew", ["update"]))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("brew", ["upgrade"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("brew", &["list"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "brew list".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| InstalledPackage {
                name: line.trim().to_string(),
                version: String::new(),
                source: PackageSource::System,
                backend: "brew".into(),
            })
            .collect();

        Ok(packages)
    }
}
