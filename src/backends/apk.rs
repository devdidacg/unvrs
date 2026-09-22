use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct ApkBackend;

impl Default for ApkBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ApkBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for ApkBackend {
    fn name(&self) -> &'static str {
        "apk"
    }

    fn is_available(&self) -> bool {
        executor::is_available("apk")
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
        let result = executor::execute("apk", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("apk search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim().to_string();
                if line.is_empty() {
                    return None;
                }
                let (name, version) = if let Some(pos) = line.rfind('-') {
                    let (n, v) = line.split_at(pos);
                    (n.to_string(), Some(v[1..].to_string()))
                } else {
                    (line, None)
                };
                Some(PackageCandidate {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "apk".into(),
                    architecture: None,
                    description: None,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("apk", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut architecture = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("描述：") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("描述:") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Description:") {
                description = Some(val.to_string());
            } else if let Some(_val) = line.strip_prefix("URL:") {
                // skip
            }
        }

        // Use apk info -a for more details
        let result2 = executor::execute("apk", &["info", "-a", package])?;
        if result2.success() {
            for line in result2.stdout.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("P: ") {
                    name = val.to_string();
                } else if let Some(val) = line.strip_prefix("V: ") {
                    version = Some(val.to_string());
                } else if let Some(val) = line.strip_prefix("A: ") {
                    architecture = Some(val.to_string());
                } else if let Some(val) = line.strip_prefix("T: ") {
                    description = Some(val.to_string());
                }
            }
        }

        if name.is_empty() {
            // Fallback: use package name
            name = package.to_string();
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "apk".into(),
            architecture,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("apk", &["add", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apk".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via apk")
            } else {
                format!(
                    "apk install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("apk", &["del", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apk".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via apk")
            } else {
                format!(
                    "apk remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("apk", &["update"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apk".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via apk".into()
            } else {
                format!(
                    "apk update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("apk", &["upgrade"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "apk".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via apk".into()
            } else {
                format!(
                    "apk upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("apk", &["list", "--installed"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "apk list --installed".into(),
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
                let name_full = parts[1].trim();
                let (name, version) = if let Some(pos) = name_full.rfind('-') {
                    let (n, v) = name_full.split_at(pos);
                    (n.to_string(), v[1..].to_string())
                } else {
                    (name_full.to_string(), String::new())
                };
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "apk".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
