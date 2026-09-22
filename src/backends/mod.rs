pub mod apk;
pub mod apt;
pub mod brew;
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
pub mod stubs;
pub mod xbps;
pub mod yum;
pub mod zypper;

use crate::config::Config;
use crate::error::Result;
use crate::package::*;

pub trait PackageManager: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn is_compatible(&self, os: &OperatingSystem) -> bool;
    fn capabilities(&self) -> BackendCapabilities;
    fn search(&self, package: &str) -> Result<Vec<PackageCandidate>>;
    fn info(&self, package: &str) -> Result<Option<PackageInfo>>;
    fn install(&self, package: &str) -> Result<InstallationResult>;
    fn remove(&self, package: &str) -> Result<InstallationResult>;
    fn update(&self) -> Result<InstallationResult>;
    fn upgrade(&self) -> Result<InstallationResult>;
    fn list_installed(&self) -> Result<Vec<InstalledPackage>>;
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
    ]
}

pub fn available_backends(os: &OperatingSystem, config: &Config) -> Vec<Box<dyn PackageManager>> {
    let preferred = config.preferred_backends();
    let mut backends: Vec<Box<dyn PackageManager>> = all_backends()
        .into_iter()
        .filter(|b| b.is_available() && b.is_compatible(os))
        .collect();

    if !preferred.is_empty() {
        backends.sort_by_key(|b| {
            let name = b.name().to_string();
            preferred
                .iter()
                .position(|p| p == &name)
                .unwrap_or(usize::MAX)
        });
    }

    backends
}
