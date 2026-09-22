use crate::backends::{available_backends, PackageManager};
use crate::config::Config;
use crate::error::Result;
use crate::package::*;

pub struct BackendRegistry {
    backends: Vec<Box<dyn PackageManager>>,
}

impl BackendRegistry {
    pub fn new(os: &OperatingSystem, config: &Config) -> Self {
        Self {
            backends: available_backends(os, config),
        }
    }

    pub fn backends(&self) -> &[Box<dyn PackageManager>] {
        &self.backends
    }

    pub fn find_by_name(&self, name: &str) -> Option<&dyn PackageManager> {
        self.backends
            .iter()
            .find(|b| b.name() == name)
            .map(|b| b.as_ref())
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
}
