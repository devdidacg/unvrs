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

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("pkg", &["install", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pkg".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via pkg")
            } else {
                format!(
                    "pkg install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("pkg", &["delete", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pkg".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via pkg")
            } else {
                format!(
                    "pkg remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("pkg", &["update"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pkg".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via pkg".into()
            } else {
                format!(
                    "pkg update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("pkg", &["upgrade", "-y"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pkg".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via pkg".into()
            } else {
                format!(
                    "pkg upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
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
