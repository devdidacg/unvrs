use crate::backends::PackageManager;
use crate::error::{Result, UnvrsError};
use crate::executor;
use crate::package::*;

macro_rules! impl_backend_stub {
    ($name:ident, $cmd:expr, $display:expr) => {
        pub struct $name;

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            pub fn new() -> Self {
                Self
            }
        }

        impl PackageManager for $name {
            fn name(&self) -> &'static str {
                $display
            }

            fn is_available(&self) -> bool {
                executor::is_available($cmd)
            }

            fn is_compatible(&self, _os: &OperatingSystem) -> bool {
                cfg!(target_os = "linux")
            }

            fn capabilities(&self) -> BackendCapabilities {
                BackendCapabilities {
                    can_search: false,
                    can_info: false,
                    can_install: false,
                    can_remove: false,
                    can_update: false,
                    can_upgrade: false,
                    can_list: false,
                }
            }

            fn search(&self, _package: &str) -> Result<Vec<PackageCandidate>> {
                Err(UnvrsError::BackendUnavailable(format!(
                    "{} backend detected but not implemented yet",
                    $display
                )))
            }

            fn info(&self, _package: &str) -> Result<Option<PackageInfo>> {
                Err(UnvrsError::BackendUnavailable(format!(
                    "{} backend detected but not implemented yet",
                    $display
                )))
            }

            fn install(&self, _package: &str) -> Result<InstallationResult> {
                Err(UnvrsError::BackendUnavailable(format!(
                    "{} backend detected but not implemented yet",
                    $display
                )))
            }

            fn remove(&self, _package: &str) -> Result<InstallationResult> {
                Err(UnvrsError::BackendUnavailable(format!(
                    "{} backend detected but not implemented yet",
                    $display
                )))
            }

            fn update(&self) -> Result<InstallationResult> {
                Err(UnvrsError::BackendUnavailable(format!(
                    "{} backend detected but not implemented yet",
                    $display
                )))
            }

            fn upgrade(&self) -> Result<InstallationResult> {
                Err(UnvrsError::BackendUnavailable(format!(
                    "{} backend detected but not implemented yet",
                    $display
                )))
            }

            fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
                Err(UnvrsError::BackendUnavailable(format!(
                    "{} backend detected but not implemented yet",
                    $display
                )))
            }
        }
    };
}

impl_backend_stub!(YumBackend, "yum", "yum");
impl_backend_stub!(ZypperBackend, "zypper", "zypper");
impl_backend_stub!(ApkBackend, "apk", "apk");
impl_backend_stub!(XbpsBackend, "xbps-install", "xbps");
impl_backend_stub!(MossBackend, "moss", "moss");
impl_backend_stub!(EmergeBackend, "emerge", "emerge");
impl_backend_stub!(EopkgBackend, "eopkg", "eopkg");
impl_backend_stub!(NixBackend, "nix", "nix");
impl_backend_stub!(GuixBackend, "guix", "guix");
impl_backend_stub!(FlatpakBackend, "flatpak", "flatpak");
impl_backend_stub!(SnapBackend, "snap", "snap");
impl_backend_stub!(PkgBackend, "pkg", "pkg");
impl_backend_stub!(BrewBackend, "brew", "brew");
