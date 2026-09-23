use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct YumBackend;

impl Default for YumBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl YumBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for YumBackend {
    fn name(&self) -> &'static str {
        "yum"
    }

    fn is_available(&self) -> bool {
        executor::is_available("yum")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux
    }

    fn native_distro_ids(&self) -> &'static [&'static str] {
        &["rhel", "centos"]
    }

    fn requires_root(&self) -> bool {
        true
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("yum", ["--version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("yum", &["search", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("yum search {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let mut candidates = Vec::new();
        let mut in_results = false;

        for line in result.stdout.lines() {
            if line.contains("==========") {
                in_results = !in_results;
                continue;
            }
            if !in_results {
                continue;
            }
            if line.starts_with("Name") || line.trim().is_empty() {
                continue;
            }
            let mut parts = line.splitn(2, " : ");
            let name = match parts.next() {
                Some(n) => n.trim().to_string(),
                None => continue,
            };
            let desc = parts.next().map(|s| s.trim().to_string());
            if !name.is_empty() {
                candidates.push(PackageCandidate {
                    name,
                    version: None,
                    source: PackageSource::System,
                    backend: "yum".into(),
                    architecture: None,
                    description: desc,
                });
            }
        }

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("yum", &["info", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut architecture = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Name        : ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version     : ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Description : ") {
                description = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Architecture: ") {
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
            backend: "yum".into(),
            architecture,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "yum",
            ["install", "-y", package],
        ))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("yum", ["remove", "-y", package]))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("yum", ["makecache"]))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("yum", ["upgrade", "-y"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("yum", &["list", "installed"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "yum list installed".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .skip(1)
            .filter(|l| !l.starts_with("Installed") && !l.starts_with("Available"))
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let name_full = parts.next()?;
                let version = parts.next()?.to_string();
                let name = name_full.split('.').next()?.to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "yum".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
