use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct PacmanBackend;

impl Default for PacmanBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PacmanBackend {
    pub fn new() -> Self {
        Self
    }

    fn aur_helper(&self) -> Option<String> {
        if executor::is_available("yay") {
            Some("yay".to_string())
        } else if executor::is_available("paru") {
            Some("paru".to_string())
        } else {
            None
        }
    }
}

impl PackageManager for PacmanBackend {
    fn name(&self) -> &'static str {
        "pacman"
    }

    fn is_available(&self) -> bool {
        executor::is_available("pacman")
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
        let result = executor::execute("pacman", &["-Ss", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("pacman -Ss {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let mut candidates = Vec::new();
        let mut current_name = String::new();
        let mut current_version = String::new();

        for line in result.stdout.lines() {
            if line.starts_with("::") || line.is_empty() {
                continue;
            }

            if let Some(pos) = line.find('/') {
                let _repo = &line[..pos];
                let rest = &line[pos + 1..];
                if let Some(space_pos) = rest.find(' ') {
                    current_name = rest[..space_pos].to_string();
                    current_version = rest[space_pos + 1..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                } else {
                    current_name = rest.to_string();
                    current_version.clear();
                }
            } else if !current_name.is_empty() && line.starts_with(' ') {
                let desc = line.trim().to_string();
                candidates.push(PackageCandidate {
                    name: current_name.clone(),
                    version: if current_version.is_empty() {
                        None
                    } else {
                        Some(current_version.clone())
                    },
                    source: PackageSource::System,
                    backend: "pacman".into(),
                    architecture: None,
                    description: Some(desc),
                });
            }
        }

        // Also search AUR if yay or paru is available
        if let Some(aur_helper) = self.aur_helper() {
            if let Ok(aur_result) = executor::execute(&aur_helper, &["-Ss", package]) {
                if aur_result.success() {
                    for line in aur_result.stdout.lines() {
                        if let Some(pos) = line.find('/') {
                            let rest = &line[pos + 1..];
                            if let Some(space_pos) = rest.find(' ') {
                                let name = rest[..space_pos].to_string();
                                let version = rest[space_pos + 1..]
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string();
                                if !candidates.iter().any(|c| c.name == name) {
                                    candidates.push(PackageCandidate {
                                        name,
                                        version: if version.is_empty() {
                                            None
                                        } else {
                                            Some(version)
                                        },
                                        source: PackageSource::System,
                                        backend: "pacman (aur)".into(),
                                        architecture: None,
                                        description: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("pacman", &["-Si", package])?;
        if !result.success() {
            if result.stderr.contains("not found") || result.exit_code == 1 {
                return Ok(None);
            }
            return Err(UnvrsError::CommandFailed {
                command: format!("pacman -Si {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut architecture = None;
        let mut maintainer = None;
        let mut homepage = None;
        let mut dependencies = Vec::new();
        let mut installed_size = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Name            : ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version         : ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Description     : ") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Architecture    : ") {
                architecture = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Maintainer      : ") {
                maintainer = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("URL             : ") {
                homepage = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Depends On      : ") {
                dependencies = val
                    .split_whitespace()
                    .filter(|s| *s != "None")
                    .map(|s| s.to_string())
                    .collect();
            } else if let Some(val) = line.strip_prefix("Installed Size  : ") {
                installed_size = Some(val.to_string());
            }
        }

        if name.is_empty() {
            return Ok(None);
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "pacman".into(),
            architecture,
            description,
            maintainer,
            homepage,
            dependencies,
            installed_size,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("pacman", &["-S", "--noconfirm", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pacman".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via pacman")
            } else {
                format!(
                    "pacman install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("pacman", &["-R", "--noconfirm", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pacman".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via pacman")
            } else {
                format!(
                    "pacman remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("pacman", &["-Sy"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pacman".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via pacman".into()
            } else {
                format!(
                    "pacman update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("pacman", &["-Su", "--noconfirm"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pacman".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via pacman".into()
            } else {
                format!(
                    "pacman upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("pacman", &["-Q"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "pacman -Q".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let name = parts.next()?.to_string();
                let version = parts.next()?.to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "pacman".into(),
                })
            })
            .collect();

        Ok(packages)
    }

    fn outdated(&self) -> Result<Vec<OutdatedPackage>> {
        let result = executor::execute("pacman", &["-Qu"])?;
        if !result.success() {
            return Ok(Vec::new());
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let name = parts.next()?.to_string();
                let current_version = parts.next()?.to_string();
                let _arrow = parts.next()?;
                let latest_version = parts.next()?.to_string();
                Some(OutdatedPackage {
                    name,
                    current_version,
                    latest_version,
                    backend: "pacman".into(),
                })
            })
            .collect();

        Ok(packages)
    }

    fn clean(&self) -> Result<InstallationResult> {
        let result = executor::execute("pacman", &["-Sc", "--noconfirm"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "pacman".into(),
            package: String::new(),
            message: if result.success() {
                "Pacman cache cleaned".into()
            } else {
                format!(
                    "pacman clean failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pacman_name() {
        let b = PacmanBackend::new();
        assert_eq!(b.name(), "pacman");
    }

    #[test]
    fn pacman_capabilities() {
        let b = PacmanBackend::new();
        let caps = b.capabilities();
        assert!(caps.can_search);
        assert!(caps.can_install);
        assert!(caps.can_remove);
    }
}
