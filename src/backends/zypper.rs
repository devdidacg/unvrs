use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct ZypperBackend;

impl Default for ZypperBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ZypperBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for ZypperBackend {
    fn name(&self) -> &'static str {
        "zypper"
    }

    fn is_available(&self) -> bool {
        executor::is_available("zypper")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux
    }

    fn native_distro_ids(&self) -> &'static [&'static str] {
        &[
            "opensuse",
            "opensuse-leap",
            "opensuse-tumbleweed",
            "sles",
            "suse",
        ]
    }

    fn requires_root(&self) -> bool {
        true
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("zypper", ["--version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("zypper", &["se", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("zypper se {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .skip_while(|l| !l.starts_with("-+"))
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(5, ' ').collect();
                if parts.len() < 5 {
                    return None;
                }
                let name = parts[4].trim().to_string();
                let version = if parts[2].trim().is_empty() {
                    None
                } else {
                    Some(parts[2].trim().to_string())
                };
                if name.is_empty() || name.starts_with("Name") || name.starts_with('-') {
                    return None;
                }
                Some(PackageCandidate {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "zypper".into(),
                    architecture: None,
                    description: None,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("zypper", &["if", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut architecture = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Name            : ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version         : ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Summary         : ") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Arch            : ") {
                architecture = Some(val.to_string());
            }
        }

        if name.is_empty() {
            return Ok(None);
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "zypper".into(),
            architecture,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("zypper", ["in", "-y", package]))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("zypper", ["rm", "-y", package]))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("zypper", ["ref"]))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("zypper", ["up", "-y"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("zypper", &["se", "-i"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "zypper se -i".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .skip_while(|l| !l.starts_with("-+"))
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(5, ' ').collect();
                if parts.len() < 5 {
                    return None;
                }
                let name = parts[4].trim().to_string();
                let version = parts[2].trim().to_string();
                if name.is_empty() || name.starts_with("Name") {
                    return None;
                }
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "zypper".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
