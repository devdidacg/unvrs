pub mod apk;
pub mod apt;
pub mod brew;
pub mod container;
pub mod dnf;
pub mod emerge;
pub mod eopkg;
pub mod flatpak;
pub mod guix;
pub mod moss;
pub mod nix;
pub mod pacman;
pub mod pkg;
pub mod snap;
pub mod xbps;
pub mod yum;
pub mod zypper;

use crate::config::Config;
use crate::error::Result;
use crate::executor::{self, CommandResult, CommandSpec};
use crate::package::*;

/// The backend interface every package manager implements.
///
/// Rules:
/// * Backend-specific knowledge (command lines, output parsing) lives here.
/// * The core never branches on backend names.
/// * `*_spec` methods are the single source of truth for mutating commands:
///   plans, dry-runs and real execution all use them.
pub trait PackageManager: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    /// OS family compatibility (search/registry filter).
    fn is_compatible(&self, os: &OperatingSystem) -> bool;
    /// Works on any distro (flatpak, snap, nix, brew, containers, ...).
    fn is_universal(&self) -> bool {
        false
    }
    /// Distro IDs (`/etc/os-release` `ID` / `ID_LIKE` values) for which this
    /// backend is the *native* package manager. Empty means "never native
    /// by distro match" (universal backends don't need it).
    fn native_distro_ids(&self) -> &'static [&'static str] {
        &[]
    }
    /// True when this backend is the native PM for the given OS.
    fn is_native_for(&self, os: &OperatingSystem) -> bool {
        if self.is_universal() {
            return true;
        }
        let ids = self.native_distro_ids();
        if ids.is_empty() {
            return false;
        }
        if ids.contains(&os.id.as_str()) {
            return true;
        }
        os.id_like.iter().any(|l| ids.contains(&l.as_str()))
    }
    /// Whether the backend's mutating operations need root/sudo.
    fn requires_root(&self) -> bool {
        false
    }
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::standard()
    }
    /// Optional version probe used by `unvrs doctor`.
    fn version_probe(&self) -> Option<CommandSpec> {
        None
    }

    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>>;
    fn info(&self, package: &str) -> Result<Option<PackageInfo>>;

    fn install_spec(&self, package: &str) -> Option<CommandSpec>;
    fn remove_spec(&self, package: &str) -> Option<CommandSpec>;
    fn update_spec(&self) -> Option<CommandSpec>;
    fn upgrade_spec(&self) -> Option<CommandSpec>;
    fn clean_spec(&self) -> Option<CommandSpec>;

    fn install(&self, package: &str) -> Result<InstallationResult> {
        match self.install_spec(package) {
            Some(spec) => run_mutation(self.name(), "install", package, &spec),
            None => Ok(unsupported(self.name(), "install")),
        }
    }

    fn remove(&self, package: &str) -> Result<InstallationResult> {
        match self.remove_spec(package) {
            Some(spec) => run_mutation(self.name(), "remove", package, &spec),
            None => Ok(unsupported(self.name(), "remove")),
        }
    }

    fn update(&self) -> Result<InstallationResult> {
        match self.update_spec() {
            Some(spec) => run_mutation(self.name(), "update", "", &spec),
            None => Ok(unsupported(self.name(), "update")),
        }
    }

    fn upgrade(&self) -> Result<InstallationResult> {
        match self.upgrade_spec() {
            Some(spec) => run_mutation(self.name(), "upgrade", "", &spec),
            None => Ok(unsupported(self.name(), "upgrade")),
        }
    }

    fn clean(&self) -> Result<InstallationResult> {
        match self.clean_spec() {
            Some(spec) => run_mutation(self.name(), "clean", "", &spec),
            None => Ok(unsupported(self.name(), "clean")),
        }
    }

    fn list_installed(&self) -> Result<Vec<InstalledPackage>>;

    fn outdated(&self) -> Result<Vec<OutdatedPackage>> {
        Ok(Vec::new())
    }
}

fn unsupported(backend: &str, op: &str) -> InstallationResult {
    InstallationResult {
        success: false,
        backend: backend.to_string(),
        package: String::new(),
        message: format!("{op} not supported by {backend}"),
        exit_code: None,
    }
}

/// Execute a mutating command spec and build a uniform result.
///
/// On permission-looking failures the message gains an actionable hint
/// instead of leaving the user with raw stderr only.
pub fn run_mutation(
    backend: &str,
    op: &str,
    package: &str,
    spec: &CommandSpec,
) -> Result<InstallationResult> {
    let result = executor::execute_mutation(&spec.program, &spec.args)?;
    let message = build_message(backend, op, package, &result);
    Ok(InstallationResult {
        success: result.success(),
        backend: backend.to_string(),
        package: package.to_string(),
        message,
        exit_code: Some(result.exit_code),
    })
}

fn build_message(backend: &str, op: &str, package: &str, result: &CommandResult) -> String {
    if result.success() {
        if package.is_empty() {
            format!("{op} completed via {backend}")
        } else {
            format!("{package} {op}ed successfully via {backend}")
        }
    } else {
        let stderr = result.stderr.trim();
        let mut msg = format!("{backend} {op} failed (exit {})", result.exit_code);
        if !stderr.is_empty() {
            let short: String = stderr.chars().take(400).collect();
            msg.push_str(&format!(": {short}"));
        }
        if looks_like_permission_error(stderr) {
            msg.push_str("\n  Reason: insufficient privileges for this operation.");
            msg.push_str(&format!("\n  Suggested action: sudo unvrs {op} {package}"));
        }
        msg
    }
}

fn looks_like_permission_error(stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();
    s.contains("permission denied")
        || s.contains("are you root")
        || s.contains("must be root")
        || s.contains("requires root")
        || s.contains("authentication is required")
        || s.contains("this operation needs to be run as root")
}

/// Probe a backend's version command; returns the first non-empty line.
pub fn probe_version(spec: &CommandSpec) -> Option<String> {
    let r = executor::execute_with_timeout(
        &spec.program,
        &spec.args,
        std::time::Duration::from_secs(10),
    )
    .ok()?;
    let text = if r.stdout.trim().is_empty() {
        &r.stderr
    } else {
        &r.stdout
    };
    text.lines()
        .map(|l| l.trim().to_string())
        .find(|l| !l.is_empty())
}

pub fn all_backends() -> Vec<Box<dyn PackageManager>> {
    vec![
        Box::new(pacman::PacmanBackend::new()),
        Box::new(apt::AptBackend::new()),
        Box::new(dnf::DnfBackend::new()),
        Box::new(yum::YumBackend::new()),
        Box::new(zypper::ZypperBackend::new()),
        Box::new(apk::ApkBackend::new()),
        Box::new(xbps::XbpsBackend::new()),
        Box::new(moss::MossBackend::new()),
        Box::new(emerge::EmergeBackend::new()),
        Box::new(eopkg::EopkgBackend::new()),
        Box::new(nix::NixBackend::new()),
        Box::new(guix::GuixBackend::new()),
        Box::new(flatpak::FlatpakBackend::new()),
        Box::new(snap::SnapBackend::new()),
        Box::new(pkg::PkgBackend::new()),
        Box::new(brew::BrewBackend::new()),
        Box::new(container::ContainerBackend::apt_docker()),
        Box::new(container::ContainerBackend::apt_podman()),
        Box::new(container::ContainerBackend::dnf_docker()),
        Box::new(container::ContainerBackend::dnf_podman()),
        Box::new(container::ContainerBackend::yum_docker()),
        Box::new(container::ContainerBackend::apk_docker()),
        Box::new(container::ContainerBackend::pacman_docker()),
        Box::new(container::ContainerBackend::zypper_docker()),
    ]
}

fn is_container_name(name: &str) -> bool {
    name.contains("(docker)") || name.contains("(podman)")
}

fn sort_by_preference(backends: &mut [Box<dyn PackageManager>], preferred: &[String]) {
    if preferred.is_empty() {
        return;
    }
    backends.sort_by_key(|b| {
        let name = b.name().to_string();
        preferred
            .iter()
            .position(|p| p == &name)
            .unwrap_or(usize::MAX)
    });
}

pub fn available_backends(os: &OperatingSystem, config: &Config) -> Vec<Box<dyn PackageManager>> {
    let preferred = config.preferred_backends();
    let mut backends: Vec<Box<dyn PackageManager>> = all_backends()
        .into_iter()
        .filter(|b| {
            if is_container_name(b.name()) {
                b.is_available()
            } else {
                b.is_available() && (b.is_compatible(os) || b.is_universal())
            }
        })
        .collect();
    sort_by_preference(&mut backends, &preferred);
    backends
}

pub fn all_backends_including_unavailable(
    os: &OperatingSystem,
    config: &Config,
) -> Vec<Box<dyn PackageManager>> {
    let preferred = config.preferred_backends();
    let mut backends: Vec<Box<dyn PackageManager>> = all_backends()
        .into_iter()
        .filter(|b| {
            if is_container_name(b.name()) {
                true
            } else {
                b.is_compatible(os) || b.is_universal()
            }
        })
        .collect();
    sort_by_preference(&mut backends, &preferred);
    backends
}

/// Known backend names for validation and suggestions.
pub fn backend_names() -> Vec<String> {
    let mut names: Vec<String> = all_backends()
        .iter()
        .map(|b| b.name().to_string())
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn linux_os(id: &str, id_like: &[&str]) -> OperatingSystem {
        OperatingSystem {
            id: id.into(),
            name: id.into(),
            version: None,
            family: OsFamily::Linux,
            arch: "x86_64".into(),
            id_like: id_like.iter().map(|s| s.to_string()).collect(),
        }
    }
    #[test]
    fn all_backends_have_unique_names() {
        let mut names: Vec<&str> = all_backends().iter().map(|b| b.name()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "duplicate backend names: {names:?}");
    }

    #[test]
    fn native_match_uses_id_and_id_like() {
        let apt = apt::AptBackend::new();
        assert!(apt.is_native_for(&linux_os("ubuntu", &[])));
        // Linux Mint: ID=linuxmint ID_LIKE="ubuntu debian"
        assert!(apt.is_native_for(&linux_os("linuxmint", &["ubuntu", "debian"])));
        assert!(!apt.is_native_for(&linux_os("arch", &["arch"])));

        let pacman = pacman::PacmanBackend::new();
        assert!(pacman.is_native_for(&linux_os("manjaro", &["arch"])));
        assert!(!pacman.is_native_for(&linux_os("fedora", &["rhel"])));
    }

    #[test]
    fn universal_backends_are_native_everywhere() {
        let fp = flatpak::FlatpakBackend::new();
        assert!(fp.is_native_for(&linux_os("fedora", &[])));
        assert!(fp.is_universal());
    }

    #[test]
    fn every_backend_declares_specs() {
        for b in all_backends() {
            assert!(
                b.install_spec("pkg").is_some(),
                "{} missing install_spec",
                b.name()
            );
            assert!(
                b.remove_spec("pkg").is_some(),
                "{} missing remove_spec",
                b.name()
            );
            assert!(
                b.update_spec().is_some(),
                "{} missing update_spec",
                b.name()
            );
            assert!(
                b.upgrade_spec().is_some(),
                "{} missing upgrade_spec",
                b.name()
            );
        }
    }

    #[test]
    fn mutation_specs_never_use_shell_meta_for_package() {
        // The package string must appear inside a single argv element (either
        // exactly, or as a channel-prefixed attribute like `nixpkgs.my-pkg`),
        // never spliced into a shell one-liner or combined with metacharacters.
        for b in all_backends() {
            for spec in [b.install_spec("my-pkg"), b.remove_spec("my-pkg")]
                .into_iter()
                .flatten()
            {
                assert!(
                    !spec.args.iter().any(|a| {
                        a.contains("sh -c")
                            || a.contains(' ')
                            || a.contains(';')
                            || a.contains('|')
                            || a.contains('$')
                            || a.contains('`')
                            || a.contains('&')
                    }),
                    "{} spec looks like shell splice: {:?}",
                    b.name(),
                    spec
                );
                assert!(
                    spec.args
                        .iter()
                        .any(|a| a == "my-pkg" || a.ends_with(".my-pkg") || a.ends_with("/my-pkg")),
                    "{} spec does not carry the package as its own arg: {:?}",
                    b.name(),
                    spec
                );
            }
        }
    }

    #[test]
    fn config_default_has_no_preference_order() {
        let cfg = Config::default();
        assert!(cfg.preferred_backends().is_empty());
    }
}
