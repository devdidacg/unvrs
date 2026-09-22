use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct YumBackend;

impl Default for YumBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl YumBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for YumBackend {
    fn name(&self) -> &'static str {
        "yum"
    }

    fn is_available(&self) -> bool {
        executor::is_available("yum")
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
        let result = executor::execute("yum", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("yum search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let mut candidates = Vec::new();
        let mut in_results = false;

        for line in result.stdout.lines() {
            if line.contains("==========") {
                in_results = !in_results;
                continue;
            }
            if !in_results {
                continue;
            }
            if line.starts_with("Name") || line.trim().is_empty() {
                continue;
            }
            let mut parts = line.splitn(2, " : ");
            let name = match parts.next() {
                Some(n) => n.trim().to_string(),
                None => continue,
            };
            let desc = parts.next().map(|s| s.trim().to_string());
            if !name.is_empty() {
                candidates.push(PackageCandidate {
                    name,
                    version: None,
                    source: PackageSource::System,
                    backend: "yum".into(),
                    architecture: None,
                    description: desc,
                });
            }
        }

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("yum", &["info", package])?;
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
            backend: "yum".into(),
            architecture,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("yum", &["install", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "yum".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via yum")
            } else {
                format!(
                    "yum install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("yum", &["remove", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "yum".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via yum")
            } else {
                format!(
                    "yum remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("yum", &["makecache"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "yum".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via yum".into()
            } else {
                format!(
                    "yum update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("yum", &["upgrade", "-y"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "yum".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via yum".into()
            } else {
                format!(
                    "yum upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("yum", &["list", "installed"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "yum list installed".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .skip(1)
            .filter(|l| !l.starts_with("Installed") && !l.starts_with("Available"))
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let name_full = parts.next()?;
                let version = parts.next()?.to_string();
                let name = name_full.split('.').next()?.to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "yum".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
