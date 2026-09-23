use crate::backends::PackageManager;
use crate::error::Result;
use crate::executor::{self, CommandSpec};
use crate::package::*;

/// Declarative description of a containerized package manager.
/// All commands are executed in exec-form (`docker run IMAGE prog args...`) —
/// never through `sh -c` with interpolated user input.
struct ContainerConfig {
    name: &'static str,
    container_engine: &'static str,
    image: &'static str,
    /// Binary + fixed args for each operation; the package name (when needed)
    /// is appended as a separate argument by the caller.
    search: (&'static str, &'static [&'static str]),
    info: (&'static str, &'static [&'static str]),
    list: (&'static str, &'static [&'static str]),
    install: (&'static str, &'static [&'static str]),
    remove: (&'static str, &'static [&'static str]),
    update: (&'static str, &'static [&'static str]),
    upgrade: (&'static str, &'static [&'static str]),
}

pub struct ContainerBackend {
    name: &'static str,
    container_engine: String,
    image: String,
    search: (&'static str, &'static [&'static str]),
    info: (&'static str, &'static [&'static str]),
    list: (&'static str, &'static [&'static str]),
    install: (&'static str, &'static [&'static str]),
    remove: (&'static str, &'static [&'static str]),
    update: (&'static str, &'static [&'static str]),
    upgrade: (&'static str, &'static [&'static str]),
}

macro_rules! container {
    ($name:expr, $engine:expr, $image:expr, $search:expr, $info:expr, $list:expr,
     $install:expr, $remove:expr, $update:expr, $upgrade:expr) => {
        ContainerBackend {
            name: $name,
            container_engine: $engine.to_string(),
            image: $image.to_string(),
            search: $search,
            info: $info,
            list: $list,
            install: $install,
            remove: $remove,
            update: $update,
            upgrade: $upgrade,
        }
    };
}

impl ContainerBackend {
    fn from_config(config: ContainerConfig) -> Self {
        container!(
            config.name,
            config.container_engine,
            config.image,
            config.search,
            config.info,
            config.list,
            config.install,
            config.remove,
            config.update,
            config.upgrade
        )
    }

    pub fn apt_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "apt (docker)",
            container_engine: "docker",
            image: "debian:latest",
            search: ("apt-cache", &["search"]),
            info: ("apt-cache", &["show"]),
            list: ("dpkg", &["--get-selections"]),
            install: ("apt-get", &["install", "-y"]),
            remove: ("apt-get", &["remove", "-y"]),
            update: ("apt-get", &["update"]),
            upgrade: ("apt-get", &["upgrade", "-y"]),
        })
    }

    pub fn apt_podman() -> Self {
        Self::from_config(ContainerConfig {
            name: "apt (podman)",
            container_engine: "podman",
            image: "debian:latest",
            search: ("apt-cache", &["search"]),
            info: ("apt-cache", &["show"]),
            list: ("dpkg", &["--get-selections"]),
            install: ("apt-get", &["install", "-y"]),
            remove: ("apt-get", &["remove", "-y"]),
            update: ("apt-get", &["update"]),
            upgrade: ("apt-get", &["upgrade", "-y"]),
        })
    }

    pub fn dnf_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "dnf (docker)",
            container_engine: "docker",
            image: "fedora:latest",
            search: ("dnf", &["search"]),
            info: ("dnf", &["info"]),
            list: ("rpm", &["-qa"]),
            install: ("dnf", &["install", "-y"]),
            remove: ("dnf", &["remove", "-y"]),
            update: ("dnf", &["makecache"]),
            upgrade: ("dnf", &["upgrade", "-y"]),
        })
    }

    pub fn dnf_podman() -> Self {
        Self::from_config(ContainerConfig {
            name: "dnf (podman)",
            container_engine: "podman",
            image: "fedora:latest",
            search: ("dnf", &["search"]),
            info: ("dnf", &["info"]),
            list: ("rpm", &["-qa"]),
            install: ("dnf", &["install", "-y"]),
            remove: ("dnf", &["remove", "-y"]),
            update: ("dnf", &["makecache"]),
            upgrade: ("dnf", &["upgrade", "-y"]),
        })
    }

    pub fn yum_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "yum (docker)",
            container_engine: "docker",
            image: "centos:7",
            search: ("yum", &["search"]),
            info: ("yum", &["info"]),
            list: ("rpm", &["-qa"]),
            install: ("yum", &["install", "-y"]),
            remove: ("yum", &["remove", "-y"]),
            update: ("yum", &["makecache"]),
            upgrade: ("yum", &["update", "-y"]),
        })
    }

    pub fn apk_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "apk (docker)",
            container_engine: "docker",
            image: "alpine:latest",
            search: ("apk", &["search"]),
            info: ("apk", &["info"]),
            list: ("apk", &["list", "--installed"]),
            install: ("apk", &["add"]),
            remove: ("apk", &["del"]),
            update: ("apk", &["update"]),
            upgrade: ("apk", &["upgrade"]),
        })
    }

    pub fn pacman_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "pacman (docker)",
            container_engine: "docker",
            image: "archlinux:latest",
            search: ("pacman", &["-Ss"]),
            info: ("pacman", &["-Si"]),
            list: ("pacman", &["-Q"]),
            install: ("pacman", &["-S", "--noconfirm"]),
            remove: ("pacman", &["-R", "--noconfirm"]),
            update: ("pacman", &["-Sy"]),
            upgrade: ("pacman", &["-Syu", "--noconfirm"]),
        })
    }

    pub fn zypper_docker() -> Self {
        Self::from_config(ContainerConfig {
            name: "zypper (docker)",
            container_engine: "docker",
            image: "opensuse/leap:latest",
            search: ("zypper", &["search"]),
            info: ("zypper", &["info"]),
            list: ("rpm", &["-qa"]),
            install: ("zypper", &["install", "-y"]),
            remove: ("zypper", &["remove", "-y"]),
            update: ("zypper", &["refresh"]),
            upgrade: ("zypper", &["update", "-y"]),
        })
    }

    /// Build the full container-engine command for an in-container operation.
    /// Exec-form only: `<engine> run --rm <image> <program> <args...>`.
    fn container_spec(&self, program: &str, args: &[&str]) -> CommandSpec {
        let mut full: Vec<String> = vec![
            "run".into(),
            "--rm".into(),
            self.image.clone(),
            program.into(),
        ];
        full.extend(args.iter().map(|a| a.to_string()));
        CommandSpec::new(&self.container_engine, full)
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

    fn requires_root(&self) -> bool {
        false
    }

    fn version_probe(&self) -> Option<CommandSpec> {
        Some(CommandSpec::new(&self.container_engine, ["--version"]))
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let (prog, fixed) = self.search;
        let mut args: Vec<&str> = fixed.to_vec();
        args.push(package);
        let spec = self.container_spec(prog, &args);
        let result = executor::execute(&spec.program, &spec.args)?;
        if !result.success() {
            return Err(crate::error::UnvrsError::CommandFailed {
                command: spec.display,
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
        let (prog, fixed) = self.info;
        let mut args: Vec<&str> = fixed.to_vec();
        args.push(package);
        let spec = self.container_spec(prog, &args);
        let result = executor::execute(&spec.program, &spec.args)?;
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

    fn install_spec(&self, package: &str) -> Option<CommandSpec> {
        let (prog, fixed) = self.install;
        let mut args: Vec<&str> = fixed.to_vec();
        args.push(package);
        Some(self.container_spec(prog, &args))
    }

    fn remove_spec(&self, package: &str) -> Option<CommandSpec> {
        let (prog, fixed) = self.remove;
        let mut args: Vec<&str> = fixed.to_vec();
        args.push(package);
        Some(self.container_spec(prog, &args))
    }

    fn update_spec(&self) -> Option<CommandSpec> {
        let (prog, fixed) = self.update;
        Some(self.container_spec(prog, fixed))
    }

    fn upgrade_spec(&self) -> Option<CommandSpec> {
        let (prog, fixed) = self.upgrade;
        Some(self.container_spec(prog, fixed))
    }

    fn clean_spec(&self) -> Option<CommandSpec> {
        None
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let (prog, fixed) = self.list;
        let spec = self.container_spec(prog, fixed);
        let result = executor::execute(&spec.program, &spec.args)?;
        if !result.success() {
            return Err(crate::error::UnvrsError::CommandFailed {
                command: spec.display,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_spec_is_exec_form_without_shell() {
        let b = ContainerBackend::apt_docker();
        let spec = b.install_spec("git").unwrap();
        assert_eq!(spec.program, "docker");
        assert_eq!(spec.args[0], "run");
        assert!(spec.args.contains(&"--rm".to_string()));
        // image then program then args — no sh -c anywhere
        assert!(!spec.args.iter().any(|a| a == "sh" || a.contains("-c")));
        assert!(spec.args.contains(&"apt-get".to_string()));
        assert!(spec.args.contains(&"git".to_string()));
    }

    #[test]
    fn hostile_package_name_stays_a_single_argument() {
        let b = ContainerBackend::apt_docker();
        let evil = "foo; rm -rf /";
        // Would be rejected by validation upstream; ensure the spec builder
        // still treats it as ONE argument (no splitting, no shell).
        let spec = b.install_spec(evil).unwrap();
        assert!(spec.args.contains(&evil.to_string()));
        assert_eq!(
            spec.args.iter().filter(|a| *a == &evil.to_string()).count(),
            1
        );
    }

    #[test]
    fn container_names_are_unique() {
        let backends = [
            ContainerBackend::apt_docker(),
            ContainerBackend::apt_podman(),
            ContainerBackend::dnf_docker(),
            ContainerBackend::dnf_podman(),
            ContainerBackend::yum_docker(),
            ContainerBackend::apk_docker(),
            ContainerBackend::pacman_docker(),
            ContainerBackend::zypper_docker(),
        ];
        let mut names: Vec<&str> = backends.iter().map(|b| b.name()).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(n, names.len());
    }
}
