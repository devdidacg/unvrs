pub mod apt;
pub mod dnf;
pub mod pacman;
pub mod stubs;

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
        Box::new(stubs::YumBackend::new()),
        Box::new(stubs::ZypperBackend::new()),
        Box::new(stubs::ApkBackend::new()),
        Box::new(stubs::XbpsBackend::new()),
        Box::new(stubs::MossBackend::new()),
        Box::new(stubs::EmergeBackend::new()),
        Box::new(stubs::EopkgBackend::new()),
        Box::new(stubs::NixBackend::new()),
        Box::new(stubs::GuixBackend::new()),
        Box::new(stubs::FlatpakBackend::new()),
        Box::new(stubs::SnapBackend::new()),
        Box::new(stubs::PkgBackend::new()),
        Box::new(stubs::BrewBackend::new()),
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
