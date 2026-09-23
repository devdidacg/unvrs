use crate::backends::{self, PackageManager};
use crate::cache;
use crate::config::Config;
use crate::error::{Result, UnvrsError};
use crate::history::{self, HistoryEntry};
use crate::os;
use crate::package::*;
use crate::plan::{self, Operation, Plan, PlanBuilder};
use crate::registry::BackendRegistry;
use crate::resolver::{ResolveOptions, Resolver};

pub struct Dispatcher {
    registry: BackendRegistry,
    os: OperatingSystem,
    config: Config,
}

/// Outcome of executing one planned action.
pub struct ExecutionOutcome {
    pub requested: String,
    pub resolved: String,
    pub backend: String,
    pub result: InstallationResult,
    pub command: String,
    pub history_id: u64,
}

/// Full resolution context returned by `explain`/`resolve`.
pub struct Resolved {
    pub resolution: crate::resolver::Resolution,
    pub source_label: String,
}

impl Dispatcher {
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

    pub fn resolver(&self) -> Resolver<'_> {
        Resolver::new(&self.registry, &self.os, &self.config)
    }

    // ---------- resolution / planning ----------

    pub fn resolve(&self, query: &str, opts: &ResolveOptions) -> Result<Resolved> {
        let resolver = self.resolver();
        let resolution = resolver.resolve(query, opts)?;
        let source_label = resolver.source_label(&resolution);
        Ok(Resolved {
            resolution,
            source_label,
        })
    }

    /// Build a complete transaction plan for `queries`. Read-only.
    pub fn build_plan(
        &self,
        operation: Operation,
        queries: &[String],
        opts: &ResolveOptions,
    ) -> Result<(Plan, Vec<crate::resolver::Resolution>)> {
        let mut resolutions = Vec::with_capacity(queries.len());
        let mut labels = Vec::with_capacity(queries.len());
        for q in queries {
            let r = self.resolve(q, opts)?;
            resolutions.push(r.resolution);
            labels.push(r.source_label);
        }

        let mut plan = PlanBuilder { os: &self.os }.build(operation, &resolutions, &labels);
        let registry = &self.registry;
        plan::attach_commands(&mut plan, |backend_name, op, pkg| {
            let backend = registry.find_by_name_any(backend_name)?;
            let spec = match op {
                Operation::Install => backend.install_spec(pkg)?,
                Operation::Remove => backend.remove_spec(pkg)?,
            };
            Some((spec, backend.requires_root()))
        });
        Ok((plan, resolutions))
    }

    /// Collect `(backend, installed names)` for every backend that can list.
    /// Used so `remove` targets the backend that actually owns the package.
    pub fn installed_map(&self) -> Vec<(String, Vec<String>)> {
        let mut map = Vec::new();
        for backend in self.registry.backends() {
            if let Ok(pkgs) = backend.list_installed() {
                map.push((
                    backend.name().to_string(),
                    pkgs.into_iter().map(|p| p.name).collect(),
                ));
            }
        }
        map
    }

    /// Execute a previously built plan. Mutates the system.
    pub fn execute_plan(&self, plan: &Plan) -> Result<Vec<ExecutionOutcome>> {
        let mut outcomes = Vec::new();

        for action in &plan.actions {
            crate::package::validate_package_id(&action.resolved)
                .map_err(UnvrsError::InvalidPackageName)?;

            let backend = self
                .registry
                .find_by_name_any(&action.backend)
                .ok_or_else(|| UnvrsError::BackendUnavailable(action.backend.clone()))?;

            let spec = match plan.operation {
                Operation::Install => backend.install_spec(&action.resolved),
                Operation::Remove => backend.remove_spec(&action.resolved),
            }
            .ok_or_else(|| {
                UnvrsError::BackendUnavailable(format!(
                    "{} does not support {:?}",
                    action.backend, plan.operation
                ))
            })?;

            crate::context::log_verbose(1, &format!("executing: {}", spec.display));
            let result = backends::run_mutation(
                backend.name(),
                plan.operation.as_str(),
                &action.resolved,
                &spec,
            )?;

            // Record a real transaction (never for dry-runs — dry-runs never
            // reach this function).
            let mut entry = HistoryEntry::now(plan.operation.as_str(), &action.requested);
            entry.backend = action.backend.clone();
            entry.package = action.resolved.clone();
            entry.resolved = action.resolved.clone();
            entry.requested = action.requested.clone();
            entry.command = spec.display.clone();
            entry.success = result.success;
            entry.exit_code = result.exit_code;
            entry.warnings = action.warnings.clone();
            entry.rollback_supported = plan.rollback_supported;
            let history_id = history::record(entry);

            outcomes.push(ExecutionOutcome {
                requested: action.requested.clone(),
                resolved: action.resolved.clone(),
                backend: action.backend.clone(),
                result,
                command: spec.display,
                history_id,
            });

            if !outcomes.last().unwrap().result.success {
                // Stop the transaction on first failure; earlier actions are
                // already recorded and reported.
                break;
            }
        }

        cache::clear_cache();
        Ok(outcomes)
    }

    // ---------- queries (unchanged behavior) ----------

    pub fn search(&self, package: &str) -> Result<Vec<PackageCandidate>> {
        let backend_refs: Vec<&dyn PackageManager> = self
            .registry
            .backends()
            .iter()
            .map(|b| b.as_ref())
            .collect();
        Ok(cache::cached_search(package, &backend_refs))
    }

    pub fn search_in_backend(
        &self,
        package: &str,
        backend_name: &str,
    ) -> Result<Vec<PackageCandidate>> {
        let known = crate::backends::backend_names();
        match self.registry.find_by_name_any(backend_name) {
            Some(backend) if backend.is_available() => backend.search(package),
            Some(_) => Err(UnvrsError::BackendUnavailable(backend_name.to_string())),
            None => Err(UnvrsError::UnknownBackend {
                name: backend_name.to_string(),
                suggestion: crate::suggest::suggest(backend_name, &known),
            }),
        }
    }

    pub fn search_with_status(&self, package: &str) -> Vec<(String, SearchStatus)> {
        self.registry
            .search_all_backends(package)
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
        cache::clear_cache();
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

    pub fn clean(&self) -> Result<Vec<InstallationResult>> {
        let mut results = Vec::new();
        for backend in self.registry.backends() {
            match backend.clean() {
                Ok(r) => results.push(r),
                Err(UnvrsError::BackendUnavailable(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }
}

#[derive(Debug)]
pub enum SearchStatus {
    Found(Vec<PackageCandidate>),
    NotFound,
    NotImplemented,
    Error,
}
