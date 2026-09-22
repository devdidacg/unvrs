use crate::cache;
use crate::config::Config;
use crate::error::{Result, UnvrsError};
use crate::os;
use crate::package::*;
use crate::registry::BackendRegistry;

pub struct Resolver {
    registry: BackendRegistry,
    os: OperatingSystem,
    config: Config,
}

impl Resolver {
    pub fn new(config: Config) -> Self {
        let os = os::detect();
        let registry = BackendRegistry::new(&os, &config);
        Self {
            registry,
            os,
            config,
        }
    }

    pub fn os(&self) -> &OperatingSystem {
        &self.os
    }

    pub fn registry(&self) -> &BackendRegistry {
        &self.registry
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let backend_refs: Vec<&dyn crate::backends::PackageManager> = self
            .registry
            .backends()
            .iter()
            .map(|b| b.as_ref())
            .collect();
        Ok(cache::cached_search(package, &backend_refs))
    }

    pub fn search_with_status(&self, package: &str) -> Vec<(String, SearchStatus)> {
        self.registry
            .search_available(package)
            .into_iter()
            .map(|(name, result)| {
                let status = match result {
                    Ok(candidates) if candidates.is_empty() => SearchStatus::NotFound,
                    Ok(candidates) => SearchStatus::Found(candidates),
                    Err(UnvrsError::BackendUnavailable(_)) => SearchStatus::NotImplemented,
                    Err(_) => SearchStatus::Error,
                };
                (name, status)
            })
            .collect()
    }

    pub fn info(&self, package: &str) -> Result<Option<PackageInfo>> {
        for backend in self.registry.backends() {
            match backend.info(package) {
                Ok(Some(info)) => return Ok(Some(info)),
                Ok(None) => continue,
                Err(UnvrsError::BackendUnavailable(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(None)
    }

    pub fn install(&self, package: &str) -> Result<InstallationResult> {
        let candidates = self.search(package)?;
        if candidates.is_empty() {
            return Err(UnvrsError::PackageNotFound(package.to_string()));
        }

        let selected = self.select_best(&candidates);
        let backend = self
            .registry
            .find_by_name(&selected.backend)
            .ok_or_else(|| UnvrsError::BackendUnavailable(selected.backend.clone()))?;

        backend.install(package)
    }

    pub fn remove(&self, package: &str) -> Result<InstallationResult> {
        for backend in self.registry.backends() {
            match backend.remove(package) {
                result @ Ok(_) => return result,
                Err(UnvrsError::BackendUnavailable(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(UnvrsError::PackageNotFound(package.to_string()))
    }

    pub fn update(&self) -> Result<Vec<InstallationResult>> {
        let mut results = Vec::new();
        for backend in self.registry.backends() {
            match backend.update() {
                Ok(r) => results.push(r),
                Err(UnvrsError::BackendUnavailable(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        cache::clear_cache();
        Ok(results)
    }

    pub fn upgrade(&self) -> Result<Vec<InstallationResult>> {
        let mut results = Vec::new();
        for backend in self.registry.backends() {
            match backend.upgrade() {
                Ok(r) => results.push(r),
                Err(UnvrsError::BackendUnavailable(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }

    pub fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let mut all = Vec::new();
        for backend in self.registry.backends() {
            match backend.list_installed() {
                Ok(mut pkgs) => all.append(&mut pkgs),
                Err(UnvrsError::BackendUnavailable(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(all)
    }

    pub fn outdated(&self) -> Result<Vec<OutdatedPackage>> {
        let mut all = Vec::new();
        for backend in self.registry.backends() {
            match backend.outdated() {
                Ok(mut pkgs) => all.append(&mut pkgs),
                Err(UnvrsError::BackendUnavailable(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(all)
    }

    fn select_best<'a>(&self, candidates: &'a [PackageCandidate]) -> &'a PackageCandidate {
        let preferred = self.config.preferred_backends();

        if !preferred.is_empty() {
            for pref in &preferred {
                if let Some(c) = candidates.iter().find(|c| &c.backend == pref) {
                    return c;
                }
            }
        }

        candidates.first().expect("called with empty candidates")
    }
}

#[derive(Debug)]
pub enum SearchStatus {
    Found(Vec<PackageCandidate>),
    NotFound,
    NotImplemented,
    Error,
}
