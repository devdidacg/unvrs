use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct EopkgBackend;

impl Default for EopkgBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl EopkgBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for EopkgBackend {
    fn name(&self) -> &'static str {
        "eopkg"
    }

    fn is_available(&self) -> bool {
        executor::is_available("eopkg")
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
        let result = executor::execute("eopkg", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("eopkg search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() < 2 {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let desc = parts[1].trim().to_string();
                if name.is_empty() || name.starts_with("Name") {
                    return None;
                }
                Some(PackageCandidate {
                    name,
                    version: None,
                    source: PackageSource::System,
                    backend: "eopkg".into(),
                    architecture: None,
                    description: Some(desc),
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("eopkg", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Name: ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version: ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Summary: ") {
                description = Some(val.to_string());
            }
        }

        if name.is_empty() {
            name = package.to_string();
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "eopkg".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("eopkg", &["install", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "eopkg".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via eopkg")
            } else {
                format!(
                    "eopkg install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("eopkg", &["remove", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "eopkg".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via eopkg")
            } else {
                format!(
                    "eopkg remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("eopkg", &["update-repo"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "eopkg".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via eopkg".into()
            } else {
                format!(
                    "eopkg update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("eopkg", &["upgrade"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "eopkg".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via eopkg".into()
            } else {
                format!(
                    "eopkg upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("eopkg", &["list-installed"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "eopkg list-installed".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() < 2 {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = parts[1].trim().to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "eopkg".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
