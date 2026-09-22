use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct MossBackend;

impl Default for MossBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MossBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for MossBackend {
    fn name(&self) -> &'static str {
        "moss"
    }

    fn is_available(&self) -> bool {
        executor::is_available("moss")
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
        let result = executor::execute("moss", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("moss search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.is_empty() {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let desc = parts.get(1).map(|s| s.trim().to_string());
                if name.is_empty() || name.starts_with("Name") {
                    return None;
                }
                Some(PackageCandidate {
                    name,
                    version: None,
                    source: PackageSource::System,
                    backend: "moss".into(),
                    architecture: None,
                    description: desc,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("moss", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = package.to_string();
        let mut version = None;
        let mut description = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Name: ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version: ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Description: ") {
                description = Some(val.to_string());
            }
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "moss".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("moss", &["install", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "moss".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via moss")
            } else {
                format!(
                    "moss install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("moss", &["remove", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "moss".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via moss")
            } else {
                format!(
                    "moss remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("moss", &["update"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "moss".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via moss".into()
            } else {
                format!(
                    "moss update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("moss", &["upgrade"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "moss".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via moss".into()
            } else {
                format!(
                    "moss upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("moss", &["list"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "moss list".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
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
                    source: PackageSource::System,
                    backend: "moss".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
