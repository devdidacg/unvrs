use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct PkgBackend;

impl Default for PkgBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PkgBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for PkgBackend {
    fn name(&self) -> &'static str {
        "pkg"
    }

    fn is_available(&self) -> bool {
        executor::is_available("pkg")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::BSD
    }

    fn native_distro_ids(&self) -> &'static [&'static str] {
        &["freebsd", "ghostbsd", "hardenedbsd", "opnsense", "pfsense"]
    }

    fn requires_root(&self) -> bool {
        true
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("pkg", ["-v"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("pkg", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("pkg search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, '-').collect();
                if parts.is_empty() {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = parts.get(1).map(|s| s.trim().to_string());
                Some(PackageCandidate {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "pkg".into(),
                    architecture: None,
                    description: None,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("pkg", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = package.to_string();
        let mut version = None;
        let mut description = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Origin: ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version: ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Comment: ") {
                description = Some(val.to_string());
            }
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "pkg".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "pkg",
            ["install", "-y", package],
        ))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("pkg", ["delete", "-y", package]))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("pkg", ["update"]))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("pkg", ["upgrade", "-y"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("pkg", &["info"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "pkg info".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, '-').collect();
                if parts.is_empty() {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = parts
                    .get(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "pkg".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
