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

    fn native_distro_ids(&self) -> &'static [&'static str] {
        &[
            "debian",
            "ubuntu",
            "raspbian",
            "devuan",
            "deepin",
            "elementary",
            "zorin",
            "pop",
            "neon",
        ]
    }

    fn requires_root(&self) -> bool {
        true
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("apt-get", ["--version"]))
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

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "apt-get",
            ["install", "-y", package],
        ))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "apt-get",
            ["remove", "-y", package],
        ))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("apt-get", ["update"]))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("apt-get", ["upgrade", "-y"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("apt-get", ["clean"]))
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
}
