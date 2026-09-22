use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct FlatpakBackend;

impl Default for FlatpakBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FlatpakBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for FlatpakBackend {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn is_available(&self) -> bool {
        executor::is_available("flatpak")
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
        let result = executor::execute("flatpak", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("flatpak search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 3 {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let description = parts.get(1).map(|s| s.trim().to_string());
                let app_id = parts.get(2).map(|s| s.trim().to_string());
                Some(PackageCandidate {
                    name: app_id.unwrap_or(name),
                    version: None,
                    source: PackageSource::Flatpak,
                    backend: "flatpak".into(),
                    architecture: None,
                    description,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("flatpak", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut homepage = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Name: ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version: ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Summary: ") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Homepage: ") {
                homepage = Some(val.to_string());
            }
        }

        if name.is_empty() {
            name = package.to_string();
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::Flatpak,
            backend: "flatpak".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("flatpak", &["install", "-y", "flathub", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "flatpak".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via flatpak")
            } else {
                format!(
                    "flatpak install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("flatpak", &["uninstall", "-y", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "flatpak".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via flatpak")
            } else {
                format!(
                    "flatpak remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("flatpak", &["update", "--appstream"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "flatpak".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via flatpak".into()
            } else {
                format!(
                    "flatpak update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("flatpak", &["update", "-y"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "flatpak".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via flatpak".into()
            } else {
                format!(
                    "flatpak upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("flatpak", &["list", "--columns=application,version"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "flatpak list".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if parts.is_empty() {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = parts.get(1).unwrap_or(&"").trim().to_string();
                if name.is_empty() {
                    return None;
                }
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::Flatpak,
                    backend: "flatpak".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
