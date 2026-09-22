use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct AptBackend;

impl Default for AptBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AptBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for AptBackend {
    fn name(&self) -> &'static str {
        "apt"
    }

    fn is_available(&self) -> bool {
        executor::is_available("apt")
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
        let result = executor::execute("apt-cache", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("apt-cache search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, " - ");
                let name = parts.next()?.trim().to_string();
                let desc = parts.next().map(|s| s.trim().to_string());
                Some(PackageCandidate {
                    name,
                    version: None,
                    source: PackageSource::System,
                    backend: "apt".into(),
                    architecture: None,
                    description: desc,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("apt-cache", &["show", package])?;
        if !result.success() {
            if result.exit_code == 0 || result.stderr.contains("No packages found") {
                return Ok(None);
            }
            return Err(UnvrsError::CommandFailed {
                command: format!("apt-cache show {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut architecture = None;
        let mut homepage = None;

        for line in result.stdout.lines() {
            if let Some(val) = line.strip_prefix("Package: ") {
                name = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("Version: ") {
                version = Some(val.trim().to_string());
            } else if let Some(val) = line.strip_prefix("Description: ") {
                description = Some(val.trim().to_string());
            } else if let Some(val) = line.strip_prefix("Architecture: ") {
                architecture = Some(val.trim().to_string());
            } else if let Some(val) = line.strip_prefix("Homepage: ") {
                homepage = Some(val.trim().to_string());
            }
        }

        if name.is_empty() {
            return Ok(None);
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "apt".into(),
            architecture,
            description,
            maintainer: None,
            homepage,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("apt-get", &["install", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apt".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via apt")
            } else {
                format!(
                    "apt install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("apt-get", &["remove", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apt".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via apt")
            } else {
                format!(
                    "apt remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("apt-get", &["update"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apt".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via apt".into()
            } else {
                format!(
                    "apt update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("apt-get", &["upgrade", "-y"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apt".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via apt".into()
            } else {
                format!(
                    "apt upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("dpkg", &["--get-selections"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "dpkg --get-selections".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter(|line| line.contains(" install"))
            .filter_map(|line| {
                let name = line.split_whitespace().next()?.to_string();
                Some(InstalledPackage {
                    name,
                    version: String::new(),
                    source: PackageSource::System,
                    backend: "apt".into(),
                })
            })
            .collect();

        Ok(packages)
    }

    fn clean(&self) -> Result<InstallationResult> {
        let result = executor::execute("apt-get", &["clean"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apt".into(),
            package: String::new(),
            message: if result.success() {
                "APT cache cleaned".into()
            } else {
                format!(
                    "apt clean failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }
}
