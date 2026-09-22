use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct GuixBackend;

impl Default for GuixBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl GuixBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for GuixBackend {
    fn name(&self) -> &'static str {
        "guix"
    }

    fn is_available(&self) -> bool {
        executor::is_available("guix")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux || os.family == OsFamily::MacOS
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
        let result = executor::execute("guix", &["package", "-A", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("guix package -A {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '\t').collect();
                if parts.len() < 2 {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = Some(parts[1].trim().to_string());
                let desc = if parts.len() > 2 {
                    Some(parts[2].trim().to_string())
                } else {
                    None
                };
                Some(PackageCandidate {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "guix".into(),
                    architecture: None,
                    description: desc,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("guix", &["package", "-A", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = package.to_string();
        let mut version = None;
        let mut description = None;

        for line in result.stdout.lines().take(1) {
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() >= 2 {
                name = parts[0].trim().to_string();
                version = Some(parts[1].trim().to_string());
            }
            if parts.len() >= 3 {
                description = Some(parts[2].trim().to_string());
            }
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "guix".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("guix", &["package", "-i", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "guix".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via guix")
            } else {
                format!(
                    "guix install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("guix", &["package", "-r", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "guix".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via guix")
            } else {
                format!(
                    "guix remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("guix", &["pull"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "guix".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via guix".into()
            } else {
                format!(
                    "guix update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("guix", &["pull"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "guix".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via guix".into()
            } else {
                format!(
                    "guix upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("guix", &["package", "-I"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "guix package -I".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 2 {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = parts[1].trim().to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "guix".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
