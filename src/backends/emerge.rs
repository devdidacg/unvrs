use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct EmergeBackend;

impl Default for EmergeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl EmergeBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for EmergeBackend {
    fn name(&self) -> &'static str {
        "emerge"
    }

    fn is_available(&self) -> bool {
        executor::is_available("emerge")
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
        let result = executor::execute("emerge", &["--search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("emerge --search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let mut candidates = Vec::new();
        let mut current_name = String::new();
        let mut current_version = String::new();

        for line in result.stdout.lines() {
            if let Some(name) = line.strip_prefix("    Package name: ") {
                current_name = name.trim().to_string();
            } else if let Some(ver) = line.strip_prefix("    Latest version available: ") {
                current_version = ver.trim().to_string();
            } else if let Some(_ver) = line.strip_prefix("    Final size installed: ") {
                if !current_name.is_empty() {
                    candidates.push(PackageCandidate {
                        name: current_name.clone(),
                        version: if current_version.is_empty() {
                            None
                        } else {
                            Some(current_version.clone())
                        },
                        source: PackageSource::System,
                        backend: "emerge".into(),
                        architecture: None,
                        description: None,
                    });
                    current_name.clear();
                    current_version.clear();
                }
            }
        }

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("emerge", &["--info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let name = package.to_string();
        let version = None;
        let description = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(_val) = line.strip_prefix("Homepage: ") {
                // we could set homepage here
            }
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "emerge".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("emerge", &[package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "emerge".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via emerge")
            } else {
                format!(
                    "emerge install failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let result = executor::execute("emerge", &["--unmerge", package])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "emerge".into(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via emerge")
            } else {
                format!(
                    "emerge remove failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = executor::execute("emerge", &["--sync"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "emerge".into(),
            package: String::new(),
            message: if result.success() {
                "Package lists updated via emerge".into()
            } else {
                format!(
                    "emerge update failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = executor::execute("emerge", &["-u", "@world"])?;
        Ok(InstallationResult {
            success: result.success(),
            backend: "emerge".into(),
            package: String::new(),
            message: if result.success() {
                "Packages upgraded via emerge".into()
            } else {
                format!(
                    "emerge upgrade failed (exit {}): {}",
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("qlist", &["-I"])?;
        if !result.success() {
            // Fallback: try portageq
            let result2 = executor::execute("portageq", &["vdb", "/", ""])?;
            if !result2.success() {
                return Err(UnvrsError::CommandFailed {
                    command: "qlist -I".into(),
                    exit_code: result.exit_code,
                    stderr: result.stderr,
                });
            }
            let packages = result2
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
                        backend: "emerge".into(),
                    })
                })
                .collect();
            return Ok(packages);
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
                    backend: "emerge".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
