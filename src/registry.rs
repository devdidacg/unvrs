use crate::backends::{all_backends_including_unavailable, available_backends, PackageManager};
use crate::config::Config;
use crate::error::Result;
use crate::package::*;

pub struct BackendRegistry {
    backends: Vec<Box<dyn PackageManager>>,
    all_backends: Vec<Box<dyn PackageManager>>,
}

impl BackendRegistry {
    pub fn new(os: &OperatingSystem, config: &Config) -> Self {
        Self {
            backends: available_backends(os, config),
            all_backends: all_backends_including_unavailable(os, config),
        }
    }

    pub fn backends(&self) -> &[Box<dyn PackageManager>] {
        &self.backends
    }

    pub fn all_backends(&self) -> &[Box<dyn PackageManager>] {
        &self.all_backends
    }

    pub fn find_by_name(&self, name: &str) -> Option<&dyn PackageManager> {
        self.backends
            .iter()
            .find(|b| b.name() == name)
            .map(|b| b.as_ref())
    }

    /// Find a backend by display label, tolerating suffixes produced by
    /// search results (e.g. `pacman (aur)` -> `pacman`).
    pub fn find_by_name_any(&self, name: &str) -> Option<&dyn PackageManager> {
        if let Some(b) = self.all_backends.iter().find(|b| b.name() == name) {
            return Some(b.as_ref());
        }
        if let Some(base) = name.strip_suffix(" (aur)") {
            if let Some(b) = self.all_backends.iter().find(|b| b.name() == base) {
                return Some(b.as_ref());
            }
        }
        None
    }

    pub fn search_all(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let mut all = Vec::new();
        for backend in &self.backends {
            if let Ok(mut candidates) = backend.search(package) {
                all.append(&mut candidates);
            }
        }
        Ok(all)
    }

    pub fn search_available(&self, package: &str) -> Vec<(String, Result<Vec<PackageCandidate>>)> {
        self.backends
            .iter()
            .map(|b| {
                let name = b.name().to_string();
                let result = b.search(package);
                (name, result)
            })
            .collect()
    }

    pub fn search_all_backends(
        &self,
        package: &str,
    ) -> Vec<(String, Result<Vec<PackageCandidate>>)> {
        self.all_backends
            .iter()
            .map(|b| {
                let name = b.name().to_string();
                let result = b.search(package);
                (name, result)
            })
            .collect()
    }

    pub fn is_native_available(&self, backend_name: &str) -> bool {
        self.backends.iter().any(|b| b.name() == backend_name)
    }
}
