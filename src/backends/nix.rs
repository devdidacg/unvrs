use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct NixBackend;

impl Default for NixBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NixBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for NixBackend {
    fn name(&self) -> &'static str {
        "nix"
    }

    fn is_available(&self) -> bool {
        executor::is_available("nix-env")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux || os.family == OsFamily::MacOS
    }

    fn is_universal(&self) -> bool {
        true
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
        let result = executor::execute("nix-env", &["-qaP", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("nix-env -qaP {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() < 2 {
                    return None;
                }
                let full_name = parts[0].trim();
                let version = parts[1].trim().to_string();
                let name = full_name.rsplit('.').next()?.to_string();
                Some(PackageCandidate {
                    name,
                    version: if version.is_empty() {
                        None
                    } else {
                        Some(version)
                    },
                    source: PackageSource::System,
                    backend: "nix".into(),
                    architecture: None,
                    description: None,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("nix-env", &["-qaP", "--description", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = package.to_string();
        let mut description = None;

        for line in result.stdout.lines() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let full_name = parts[0].trim();
                let desc = parts[1].trim();
                name = full_name
                    .rsplit('.')
                    .next()
                    .unwrap_or(full_name)
                    .to_string();
                if !desc.is_empty() {
                    description = Some(desc.to_string());
                }
                break;
            }
        }

        Ok(Some(PackageInfo {
            name,
            version: None,
            source: PackageSource::System,
            backend: "nix".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("nix-env", &["-iA", &format!("nixpkgs.{package}")])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "nix".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via nix")
            } else {
                format!(
                    "nix install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("nix-env", &["-e", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "nix".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via nix")
            } else {
                format!(
                    "nix remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("nix-channel", &["--update"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "nix".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via nix".into()
            } else {
                format!(
                    "nix update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("nix-env", &["-u"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "nix".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via nix".into()
            } else {
                format!(
                    "nix upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("nix-env", &["-q"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "nix-env -q".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim().to_string();
                if line.is_empty() {
                    return None;
                }
                let (name, version) = if let Some(pos) = line.rfind('-') {
                    let (n, v) = line.split_at(pos);
                    (n.to_string(), v[1..].to_string())
                } else {
                    (line, String::new())
                };
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "nix".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
