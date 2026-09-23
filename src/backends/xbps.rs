use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct XbpsBackend;

impl Default for XbpsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl XbpsBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for XbpsBackend {
    fn name(&self) -> &'static str {
        "xbps"
    }

    fn is_available(&self) -> bool {
        executor::is_available("xbps-install")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux
    }

    fn native_distro_ids(&self) -> &'static [&'static str] {
        &["void"]
    }

    fn requires_root(&self) -> bool {
        true
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("xbps-install", ["--version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("xbps-install", &["-Ss", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("xbps-install -Ss {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim().to_string();
                if line.is_empty() || line.starts_with('I') || line.starts_with("Name") {
                    return None;
                }
                let mut parts = line.split_whitespace();
                let name = parts.next()?.to_string();
                let version = parts.next().map(|s| s.to_string());
                Some(PackageCandidate {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "xbps".into(),
                    architecture: None,
                    description: parts.collect::<Vec<_>>().join(" ").into(),
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("xbps-install", &["-Si", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = String::new();
        let mut version = None;
        let mut description = None;
        let mut architecture = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("pkgver: ") {
                let parts: Vec<&str> = val.splitn(2, '-').collect();
                name = parts[0].to_string();
                if parts.len() > 1 {
                    version = Some(parts[1].to_string());
                }
            } else if let Some(val) = line.strip_prefix("short_desc: ") {
                description = Some(val.trim().to_string());
            } else if let Some(val) = line.strip_prefix("arch: ") {
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
            backend: "xbps".into(),
            architecture,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("xbps-install", ["-y", package]))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("xbps-remove", ["-y", package]))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("xbps-install", ["-Su"]))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("xbps-install", ["-Su"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("xbps-query", &["-l"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "xbps-query -l".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() < 3 {
                    return None;
                }
                let name = parts[2].split('-').next()?.to_string();
                let version = parts[1].to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "xbps".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
