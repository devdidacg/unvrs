use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct DnfBackend;

impl Default for DnfBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl DnfBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for DnfBackend {
    fn name(&self) -> &'static str {
        "dnf"
    }

    fn is_available(&self) -> bool {
        executor::is_available("dnf")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            can_search: true,
            can_info: true,
            can_install: true,
            can_remove: true,
            can_update: true,
            can_upgrade: true,
            can_list: true,
        }
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("dnf", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("dnf search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .skip_while(|l| !l.contains("=========="))
            .skip(1)
            .filter_map(|line| {
                let mut parts = line.splitn(2, " : ");
                let name = parts.next()?.trim().to_string();
                let desc = parts.next().map(|s| s.trim().to_string());
                if name.is_empty() || name.starts_with("Name") || name.starts_with('=') {
                    return None;
                }
                Some(PackageCandidate {
                    name,
                    version: None,
                    source: PackageSource::System,
                    backend: "dnf".into(),
                    architecture: None,
                    description: desc,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("dnf", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut architecture = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Name        : ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version     : ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Description : ") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Architecture: ") {
                architecture = Some(val.to_string());
            }
        }

        if name.is_empty() {
            return Ok(None);
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "dnf".into(),
            architecture,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("dnf", &["install", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "dnf".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via dnf")
            } else {
                format!(
                    "dnf install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("dnf", &["remove", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "dnf".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via dnf")
            } else {
                format!(
                    "dnf remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("dnf", &["makecache"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "dnf".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via dnf".into()
            } else {
                format!(
                    "dnf update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("dnf", &["upgrade", "-y"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "dnf".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via dnf".into()
            } else {
                format!(
                    "dnf upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("dnf", &["list", "installed"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "dnf list installed".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .skip(1)
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let name_full = parts.next()?;
                let version = parts.next()?.to_string();
                let name = name_full.split('.').next()?.to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "dnf".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
