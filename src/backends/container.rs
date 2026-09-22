use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor::{self, CommandResult};
use crate::package::*;

struct ContainerConfig {
    name: &'static str,
    container_engine: &'static str,
    image: &'static str,
    install_prefix: &'static str,
    remove_prefix: &'static str,
    search_prefix: &'static str,
    info_prefix: &'static str,
    list_prefix: &'static str,
    update_prefix: &'static str,
    upgrade_prefix: &'static str,
}

pub struct ContainerBackend {
    name: &'static str,
    container_engine: String,
    image: String,
    install_prefix: String,
    remove_prefix: String,
    search_prefix: String,
    info_prefix: String,
    list_prefix: String,
    update_prefix: String,
    upgrade_prefix: String,
}

impl ContainerBackend {
    fn from_config(config: ContainerConfig) -> Self {
        Self {
            name: config.name,
            container_engine: config.container_engine.to_string(),
            image: config.image.to_string(),
            install_prefix: config.install_prefix.to_string(),
            remove_prefix: config.remove_prefix.to_string(),
            search_prefix: config.search_prefix.to_string(),
            info_prefix: config.info_prefix.to_string(),
            list_prefix: config.list_prefix.to_string(),
            update_prefix: config.update_prefix.to_string(),
            upgrade_prefix: config.upgrade_prefix.to_string(),
        }
    }

    pub fn apt_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "apt (docker)",
            container_engine: "docker",
            image: "debian:latest",
            install_prefix: "apt-get install -y",
            remove_prefix: "apt-get remove -y",
            search_prefix: "apt-cache search",
            info_prefix: "apt-cache show",
            list_prefix: "dpkg -l",
            update_prefix: "apt-get update",
            upgrade_prefix: "apt-get upgrade -y",
        })
    }

    pub fn apt_podman() -> Self {
        Self::from_config(ContainerConfig {
            name: "apt (podman)",
            container_engine: "podman",
            image: "debian:latest",
            install_prefix: "apt-get install -y",
            remove_prefix: "apt-get remove -y",
            search_prefix: "apt-cache search",
            info_prefix: "apt-cache show",
            list_prefix: "dpkg -l",
            update_prefix: "apt-get update",
            upgrade_prefix: "apt-get upgrade -y",
        })
    }

    pub fn dnf_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "dnf (docker)",
            container_engine: "docker",
            image: "fedora:latest",
            install_prefix: "dnf install -y",
            remove_prefix: "dnf remove -y",
            search_prefix: "dnf search",
            info_prefix: "dnf info",
            list_prefix: "rpm -qa",
            update_prefix: "dnf check-update || true",
            upgrade_prefix: "dnf upgrade -y",
        })
    }

    pub fn dnf_podman() -> Self {
        Self::from_config(ContainerConfig {
            name: "dnf (podman)",
            container_engine: "podman",
            image: "fedora:latest",
            install_prefix: "dnf install -y",
            remove_prefix: "dnf remove -y",
            search_prefix: "dnf search",
            info_prefix: "dnf info",
            list_prefix: "rpm -qa",
            update_prefix: "dnf check-update || true",
            upgrade_prefix: "dnf upgrade -y",
        })
    }

    pub fn yum_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "yum (docker)",
            container_engine: "docker",
            image: "centos:7",
            install_prefix: "yum install -y",
            remove_prefix: "yum remove -y",
            search_prefix: "yum search",
            info_prefix: "yum info",
            list_prefix: "rpm -qa",
            update_prefix: "yum check-update || true",
            upgrade_prefix: "yum update -y",
        })
    }

    pub fn apk_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "apk (docker)",
            container_engine: "docker",
            image: "alpine:latest",
            install_prefix: "apk add",
            remove_prefix: "apk del",
            search_prefix: "apk search",
            info_prefix: "apk info",
            list_prefix: "apk list --installed",
            update_prefix: "apk update",
            upgrade_prefix: "apk upgrade",
        })
    }

    pub fn pacman_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "pacman (docker)",
            container_engine: "docker",
            image: "archlinux:latest",
            install_prefix: "pacman -S --noconfirm",
            remove_prefix: "pacman -R --noconfirm",
            search_prefix: "pacman -Ss",
            info_prefix: "pacman -Si",
            list_prefix: "pacman -Q",
            update_prefix: "pacman -Sy",
            upgrade_prefix: "pacman -Syu --noconfirm",
        })
    }

    pub fn zypper_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "zypper (docker)",
            container_engine: "docker",
            image: "opensuse/leap:latest",
            install_prefix: "zypper install -y",
            remove_prefix: "zypper remove -y",
            search_prefix: "zypper search",
            info_prefix: "zypper info",
            list_prefix: "rpm -qa",
            update_prefix: "zypper refresh",
            upgrade_prefix: "zypper update -y",
        })
    }

    fn run_in_container(&self, command: &str) -> Result<CommandResult> {
        let safe_cmd = command.replace('"', "\\\"");
        let full_command = format!(
            "{engine} run --rm --name unvrs-{pid} {image} sh -c \"{cmd}\"",
            engine = self.container_engine,
            pid = std::process::id(),
            image = self.image,
            cmd = safe_cmd,
        );

        executor::execute_raw(&full_command)
    }
}

impl PackageManager for ContainerBackend {
    fn name(&self) -> &'static str {
        self.name
    }

    fn is_available(&self) -> bool {
        executor::is_available(&self.container_engine)
    }

    fn is_compatible(&self, _os: &OperatingSystem) -> bool {
        false
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
        let command = format!("{} {}", self.search_prefix, package);
        let result = self.run_in_container(&command)?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("{} search {}", self.name, package),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }

        let candidates = result
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.is_empty() {
                    return None;
                }
                let name = parts[0].trim().to_string();
                let description = parts.get(1).map(|s| s.trim().to_string());
                Some(PackageCandidate {
                    name,
                    version: None,
                    source: PackageSource::System,
                    backend: self.name.to_string(),
                    architecture: None,
                    description,
                })
            })
            .collect();

        Ok(candidates)
    }

    fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        let command = format!("{} {}", self.info_prefix, package);
        let result = self.run_in_container(&command)?;
        if !result.success() {
            return Ok(None);
        }

        let mut name = package.to_string();
        let mut version = None;
        let mut description = None;

        for line in result.stdout.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("Package: ") {
                name = val.to_string();
            } else if let Some(val) = line.strip_prefix("Version: ") {
                version = Some(val.to_string());
            } else if let Some(val) = line.strip_prefix("Description: ") {
                description = Some(val.to_string());
            }
        }

        Ok(Some(PackageInfo {
            name,
            version,
            source: PackageSource::System,
            backend: self.name.to_string(),
            architecture: None,
            description,
            maintainer: None,
            homepage: None,
            dependencies: Vec::new(),
            installed_size: None,
        }))
    }

    fn install(&self, package: &str) -> Result<InstallationResult> {
        let command = format!("{} {}", self.install_prefix, package);
        let result = self.run_in_container(&command)?;
        Ok(InstallationResult {
            success: result.success(),
            backend: self.name.to_string(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} installed successfully via {}", self.name)
            } else {
                format!(
                    "{} install failed (exit {}): {}",
                    self.name,
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        let command = format!("{} {}", self.remove_prefix, package);
        let result = self.run_in_container(&command)?;
        Ok(InstallationResult {
            success: result.success(),
            backend: self.name.to_string(),
            package: package.to_string(),
            message: if result.success() {
                format!("{package} removed successfully via {}", self.name)
            } else {
                format!(
                    "{} remove failed (exit {}): {}",
                    self.name,
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn update(&self) -> Result<InstallationResult> {
        let result = self.run_in_container(&self.update_prefix)?;
        Ok(InstallationResult {
            success: result.success(),
            backend: self.name.to_string(),
            package: String::new(),
            message: if result.success() {
                format!("Package lists updated via {}", self.name)
            } else {
                format!(
                    "{} update failed (exit {}): {}",
                    self.name,
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        let result = self.run_in_container(&self.upgrade_prefix)?;
        Ok(InstallationResult {
            success: result.success(),
            backend: self.name.to_string(),
            package: String::new(),
            message: if result.success() {
                format!("Packages upgraded via {}", self.name)
            } else {
                format!(
                    "{} upgrade failed (exit {}): {}",
                    self.name,
                    result.exit_code,
                    result.stderr.trim()
                )
            },
        })
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let result = self.run_in_container(&self.list_prefix)?;
        if !result.success() {
            return Err(UnvrsError::CommandFailed {
                command: format!("{} list", self.name),
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
                let (name, version) = if let Some(pos) = line.find(char::is_whitespace) {
                    let (n, v) = line.split_at(pos);
                    (n.to_string(), v.trim().to_string())
                } else {
                    (line, String::new())
                };
                Some(InstalledPackage {
                    name,
                    version,
                    source: PackageSource::System,
                    backend: self.name.to_string(),
                })
            })
            .collect();

        Ok(packages)
    }
}
