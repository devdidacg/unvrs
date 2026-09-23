use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

pub struct GuixBackend;

impl Default for GuixBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl GuixBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for GuixBackend {
    fn name(&self) -> &'static str {
        "guix"
    }

    fn is_available(&self) -> bool {
        executor::is_available("guix")
    }

    fn is_compatible(&self, os: &OperatingSystem) -> bool {
        os.family == OsFamily::Linux || os.family == OsFamily::MacOS
    }

    fn is_universal(&self) -> bool {
        true
    }

    fn requires_root(&self) -> bool {
        false
    }

    fn version_probe(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("guix", ["--version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let result = executor::execute("guix", &["package", "-A", package])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("guix package -A {package}"),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '\t').collect();
                if parts.len() < 2 {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = Some(parts[1].trim().to_string());
                let desc = if parts.len() > 2 {
                    Some(parts[2].trim().to_string())
                } else {
                    None
                };
                Some(PackageCandidate {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "guix".into(),
                    architecture: None,
                    description: desc,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let result = executor::execute("guix", &["package", "-A", package])?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = package.to_string();
        let mut version = None;
        let mut description = None;

        for line in result.stdout.lines().take(1) {
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() >= 2 {
                name = parts[0].trim().to_string();
                version = Some(parts[1].trim().to_string());
            }
            if parts.len() >= 3 {
                description = Some(parts[2].trim().to_string());
            }
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: "guix".into(),
            architecture: None,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "guix",
            ["package", "-i", package],
        ))
    }

    fn remove_spec(&self, package: &str) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new(
            "guix",
            ["package", "-r", package],
        ))
    }

    fn update_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("guix", ["pull"]))
    }

    fn upgrade_spec(&self) -> Option<executor::CommandSpec> {
        Some(executor::CommandSpec::new("guix", ["pull"]))
    }

    fn clean_spec(&self) -> Option<executor::CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = executor::execute("guix", &["package", "-I"])?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: "guix package -I".into(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let packages = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 2 {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let version = parts[1].trim().to_string();
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: "guix".into(),
                })
            })
            .collect();

        Ok(packages)
    }
}
