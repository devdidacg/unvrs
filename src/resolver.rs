use crate::backends::PackageManager;
use crate::config::{Config, KEYWORD_CONTAINER, KEYWORD_NATIVE, KEYWORD_UNIVERSAL};
use crate::error::{Result, UnvrsError};
use crate::package::*;
use crate::registry::BackendRegistry;

/// How a backend relates to the current system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum BackendClass {
    Native,
    Universal,
    Container,
    /// Non-native PM executing directly on the host (needs `--cross-distro`).
    Cross,
}

impl BackendClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackendClass::Native => "native",
            BackendClass::Universal => "universal",
            BackendClass::Container => "container",
            BackendClass::Cross => "cross-distro",
        }
    }
}

impl std::fmt::Display for BackendClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CandidateReport {
    pub backend: String,
    pub class: BackendClass,
    pub available: bool,
    pub compatible: bool,
    pub native: bool,
    pub universal: bool,
    pub status: String,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub requested: String,
    pub resolved_name: String,
    pub backend_name: String,
    pub class: BackendClass,
    pub candidate: PackageCandidate,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub report: Vec<CandidateReport>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    pub explicit_backend: Option<String>,
    pub cross_distro: bool,
    pub prefer_container: bool,
    /// `(backend, installed names)` pairs; when set, only backends that
    /// report the package installed may be selected (used by `remove`).
    pub installed_packages: Option<Vec<(String, Vec<String>)>>,
}

fn classify(backend: &dyn PackageManager, os: &OperatingSystem, native: bool) -> BackendClass {
    let name = backend.name();
    if name.contains("(docker)") || name.contains("(podman)") {
        return BackendClass::Container;
    }
    if backend.is_universal() {
        return BackendClass::Universal;
    }
    if native && backend.is_compatible(os) {
        return BackendClass::Native;
    }
    BackendClass::Cross
}

/// Policy rank of a backend: index of the first `prefer` entry matching either
/// the exact backend name or its class keyword. Unlisted entries get `usize::MAX`.
fn rank_in_policy(prefer: &[String], name: &str, class: BackendClass) -> usize {
    let keyword = match class {
        BackendClass::Native => KEYWORD_NATIVE,
        BackendClass::Universal => KEYWORD_UNIVERSAL,
        BackendClass::Container => KEYWORD_CONTAINER,
        BackendClass::Cross => "cross-distro",
    };
    prefer
        .iter()
        .position(|p| p == name || p == keyword)
        .unwrap_or(usize::MAX)
}

/// Choose the concrete package name: prefer exact query match, else first hit.
fn pick_candidate(candidates: &[PackageCandidate], query: &str) -> PackageCandidate {
    candidates
        .iter()
        .find(|c| c.name == query)
        .or_else(|| candidates.first())
        .cloned()
        .expect("resolve called with empty candidate list")
}

pub struct Resolver<'a> {
    registry: &'a BackendRegistry,
    os: &'a OperatingSystem,
    config: &'a Config,
}

impl<'a> Resolver<'a> {
    pub fn new(registry: &'a BackendRegistry, os: &'a OperatingSystem, config: &'a Config) -> Self {
        Self {
            registry,
            os,
            config,
        }
    }

    /// Deterministically resolve `query` to a single backend + package name.
    /// Never executes mutating commands.
    pub fn resolve(&self, query: &str, opts: &ResolveOptions) -> Result<Resolution> {
        crate::package::validate_package_id(query).map_err(UnvrsError::InvalidPackageName)?;

        if let Some(name) = &opts.explicit_backend {
            return self.resolve_explicit(query, name, opts);
        }
        self.resolve_auto(query, opts)
    }

    fn resolve_auto(&self, query: &str, opts: &ResolveOptions) -> Result<Resolution> {
        let all = self.registry.all_backends();
        let avoid = self.config.avoid();
        let prefer = self.config.effective_prefer();
        let prefer_native = self.config.prefer_current_distro();

        struct Scored<'b> {
            backend: &'b dyn PackageManager,
            candidates: Vec<PackageCandidate>,
            class: BackendClass,
            policy_rank: usize,
            native_rank: usize,
            exact_match: bool,
            registration_index: usize,
        }

        let mut report: Vec<CandidateReport> = Vec::new();
        let mut scored: Vec<Scored> = Vec::new();
        let native_found_any = all
            .iter()
            .any(|b| b.is_available() && b.is_native_for(self.os));

        for (idx, backend) in all.iter().enumerate() {
            let available = backend.is_available();
            let universal = backend.is_universal();
            let native = backend.is_native_for(self.os);
            let compatible = backend.is_compatible(self.os) || universal;
            let class = classify(backend.as_ref(), self.os, native);

            let mut status;
            let mut note = String::new();
            let mut candidates = Vec::new();

            if !available {
                status = "unavailable";
                note = "backend not installed".into();
            } else if !compatible && class != BackendClass::Container {
                status = "incompatible";
                note = format!("not compatible with {}", self.os.family);
            } else {
                match backend.search(query) {
                    Ok(found) if found.is_empty() => {
                        status = "no-results";
                        note = "package not found in this backend".into();
                    }
                    Ok(found) => {
                        candidates = found;
                        if let Some(installed) = &opts.installed_packages {
                            if let Some((_, names)) =
                                installed.iter().find(|(b, _)| b == backend.name())
                            {
                                if !names.iter().any(|n| n == query) {
                                    status = "not-installed";
                                    note = "package not installed via this backend".into();
                                } else {
                                    status = "candidate";
                                }
                            } else {
                                status = "not-installed";
                                note = "backend did not report an installed copy".into();
                            }
                        } else {
                            status = "candidate";
                        }
                        if status == "candidate" && avoid.iter().any(|a| a == backend.name()) {
                            status = "avoided";
                            note = "listed in policy.avoid".into();
                        } else if status == "candidate"
                            && class == BackendClass::Cross
                            && !opts.cross_distro
                        {
                            status = "cross-distro-not-allowed";
                            note = "requires --cross-distro".into();
                        }
                    }
                    Err(e) => {
                        status = "error";
                        note = e.to_string();
                    }
                }
            }

            report.push(CandidateReport {
                backend: backend.name().to_string(),
                class,
                available,
                compatible,
                native,
                universal,
                status: status.to_string(),
                note,
            });

            if status == "candidate" {
                let policy_rank = rank_in_policy(&prefer, backend.name(), class);
                let effective_rank = if opts.prefer_container && class == BackendClass::Container {
                    0
                } else {
                    policy_rank
                };
                let native_rank = if class == BackendClass::Native && prefer_native {
                    0
                } else if class == BackendClass::Universal {
                    1
                } else if class == BackendClass::Container {
                    2
                } else {
                    3
                };
                let exact_match = candidates.iter().any(|c| c.name == query);
                scored.push(Scored {
                    backend: backend.as_ref(),
                    candidates,
                    class,
                    policy_rank: effective_rank,
                    native_rank,
                    exact_match,
                    registration_index: idx,
                });
            }
        }

        if scored.is_empty() {
            if !report.iter().any(|r| r.available) {
                return Err(UnvrsError::BackendUnavailable(
                    "no compatible backends available on this system".into(),
                ));
            }
            // If every available backend failed to run (daemon down, network
            // error, ...), say so instead of pretending the package is absent.
            let definitive = report.iter().any(|r| {
                matches!(
                    r.status.as_str(),
                    "no-results" | "not-installed" | "avoided" | "cross-distro-not-allowed"
                )
            });
            if !definitive {
                let errors: Vec<String> = report
                    .iter()
                    .filter(|r| r.status == "error")
                    .map(|r| format!("{}: {}", r.backend, r.note))
                    .collect();
                if !errors.is_empty() {
                    return Err(UnvrsError::BackendUnavailable(format!(
                        "backend search failed for `{query}` — {}",
                        errors.join("; ")
                    )));
                }
            }
            if opts.installed_packages.is_some() {
                return Err(UnvrsError::PackageNotFound(format!(
                    "{query} (not reported as installed by any backend)"
                )));
            }
            let any_cross = report
                .iter()
                .any(|r| r.status == "cross-distro-not-allowed");
            if any_cross && !opts.cross_distro {
                return Err(UnvrsError::RequiresCrossDistro(format!(
                    "{query} is only available from a non-native backend"
                )));
            }
            return Err(UnvrsError::PackageNotFound(query.to_string()));
        }

        // Deterministic: policy rank, class preference, exact match, class,
        // registration order.
        scored.sort_by_key(|s| {
            (
                s.policy_rank,
                s.native_rank,
                !s.exact_match as u8,
                s.class,
                s.registration_index,
            )
        });

        let best = &scored[0];
        let candidate = pick_candidate(&best.candidates, query);

        for r in report.iter_mut() {
            if r.backend == best.backend.name() && r.status == "candidate" {
                r.status = "selected".into();
            }
        }

        let mut reasons = vec![match best.class {
            BackendClass::Native => format!(
                "native package manager for {} ({})",
                self.os.label(),
                self.os.id
            ),
            BackendClass::Universal => "universal backend (works on this OS)".to_string(),
            BackendClass::Container => "isolated container backend".to_string(),
            BackendClass::Cross => "non-native backend (explicitly allowed)".to_string(),
        }];
        reasons.push(format!(
            "policy priority: {}",
            if best.policy_rank == usize::MAX {
                "unlisted (fallback)".to_string()
            } else {
                format!("position {} in prefer list", best.policy_rank + 1)
            }
        ));
        if !native_found_any && best.class != BackendClass::Native {
            reasons.push("no native backend detected for this OS; fell back by policy".into());
        }
        if best.exact_match {
            reasons.push("exact package name match".into());
        }
        reasons.push("package available in this backend".into());

        let mut warnings = Vec::new();
        match best.class {
            BackendClass::Cross => warnings.push(format!(
                "{} is not the native package manager for {}; cross-distro operation",
                best.backend.name(),
                self.os.label()
            )),
            BackendClass::Container => warnings.push(format!(
                "package will be installed inside a container via {}",
                best.backend.name()
            )),
            _ => {}
        }
        if best.backend.requires_root() {
            warnings.push("requires elevated privileges (root/sudo)".into());
        }

        Ok(Resolution {
            requested: query.to_string(),
            resolved_name: candidate.name.clone(),
            backend_name: best.backend.name().to_string(),
            class: best.class,
            candidate,
            reasons,
            warnings,
            report,
        })
    }

    fn resolve_explicit(
        &self,
        query: &str,
        name: &str,
        opts: &ResolveOptions,
    ) -> Result<Resolution> {
        let known = crate::backends::backend_names();
        let backend = self.registry.find_by_name_any(name).ok_or_else(|| {
            if known.iter().any(|k| k == name) {
                UnvrsError::BackendUnavailable(name.to_string())
            } else {
                UnvrsError::UnknownBackend {
                    name: name.to_string(),
                    suggestion: crate::suggest::suggest(name, &known),
                }
            }
        })?;

        if !backend.is_available() {
            return Err(UnvrsError::BackendUnavailable(name.to_string()));
        }

        // Remove must target a package this backend reports as installed.
        if let Some(installed) = &opts.installed_packages {
            match installed.iter().find(|(b, _)| b == backend.name()) {
                Some((_, names)) if names.iter().any(|n| n == query) => {}
                _ => {
                    return Err(UnvrsError::PackageNotFound(format!(
                        "{query} (not reported as installed by {name})"
                    )));
                }
            }
        }

        let native = backend.is_native_for(self.os);
        let class = classify(backend, self.os, native);

        // Safety gate: direct-execution non-native backends need --cross-distro.
        if class == BackendClass::Cross && !opts.cross_distro {
            return Err(UnvrsError::RequiresCrossDistro(format!(
                "`{name}` is not the native package manager for {}",
                self.os.label()
            )));
        }

        let candidates = backend.search(query)?;
        if candidates.is_empty() {
            if opts.installed_packages.is_some() {
                return Err(UnvrsError::PackageNotFound(format!(
                    "{query} (not found in {name})"
                )));
            }
            return Err(UnvrsError::PackageNotFound(format!("{query} (in {name})")));
        }
        let candidate = pick_candidate(&candidates, query);

        let mut report = Vec::new();
        for b in self.registry.all_backends() {
            let b_native = b.is_native_for(self.os);
            let b_class = classify(b.as_ref(), self.os, b_native);
            let selected = b.name() == backend.name();
            report.push(CandidateReport {
                backend: b.name().to_string(),
                class: b_class,
                available: b.is_available(),
                compatible: b.is_compatible(self.os) || b.is_universal(),
                native: b_native,
                universal: b.is_universal(),
                status: if selected {
                    "selected".into()
                } else if !b.is_available() {
                    "unavailable".into()
                } else {
                    "not-requested".into()
                },
                note: if selected {
                    "explicitly requested via --backend".into()
                } else {
                    String::new()
                },
            });
        }

        let mut warnings = Vec::new();
        if class == BackendClass::Cross {
            warnings.push(format!(
                "`{name}` is not the native package manager for {}; cross-distro operation",
                self.os.label()
            ));
        }
        if class == BackendClass::Container {
            warnings.push(format!(
                "package will be installed inside a container via {name}"
            ));
        }
        if backend.requires_root() {
            warnings.push("requires elevated privileges (root/sudo)".into());
        }

        Ok(Resolution {
            requested: query.to_string(),
            resolved_name: candidate.name.clone(),
            backend_name: backend.name().to_string(),
            class,
            candidate,
            reasons: vec![
                format!("explicitly requested via --backend {name}"),
                match class {
                    BackendClass::Native => "also the native backend for this OS".to_string(),
                    BackendClass::Universal => "universal backend".to_string(),
                    BackendClass::Container => "isolated container backend".to_string(),
                    BackendClass::Cross => {
                        "cross-distro backend (allowed by --cross-distro)".to_string()
                    }
                },
                "package available in this backend".into(),
            ],
            warnings,
            report,
        })
    }

    /// One-line source label for plans, e.g. "Ubuntu repository (apt)".
    pub fn source_label(&self, res: &Resolution) -> String {
        match res.class {
            BackendClass::Native => {
                format!("{} repository ({})", self.os.label(), res.backend_name)
            }
            BackendClass::Universal => format!("{} (universal source)", res.backend_name),
            BackendClass::Container => format!("containerized {}", res.backend_name),
            BackendClass::Cross => format!("{} (cross-distro)", res.backend_name),
        }
    }
}

/// Render the `--explain` view: environment, per-backend candidates,
/// selection and reasons.
pub fn render_explain(os: &OperatingSystem, res: &Resolution) -> String {
    let mut out = String::new();
    out.push_str("Detected system:\n");
    out.push_str(&format!("  {}\n", os.label()));
    out.push_str(&format!("  {} ({})\n\n", os.arch, os.family));

    out.push_str("Candidates:\n");
    let width = res
        .report
        .iter()
        .map(|r| r.backend.len())
        .max()
        .unwrap_or(6);
    for r in &res.report {
        let mark = match r.status.as_str() {
            "selected" => "selected",
            "candidate" => "available",
            "unavailable" => "unavailable",
            "no-results" => "no package",
            "avoided" => "avoided",
            "cross-distro-not-allowed" => "needs --cross-distro",
            "not-installed" => "not installed",
            "error" => "error",
            other => other,
        };
        out.push_str(&format!(
            "  {:<width$}  {:<20} ({})\n",
            r.backend,
            mark,
            r.class,
            width = width
        ));
        if !r.note.is_empty() && r.status != "selected" {
            out.push_str(&format!("  {:<width$}    {}\n", "", r.note, width = width));
        }
    }

    out.push_str("\nSelected:\n");
    out.push_str(&format!("  {}\n", res.backend_name));
    out.push_str(&format!("  resolved package: {}\n", res.resolved_name));

    out.push_str("\nReason:\n");
    for r in &res.reasons {
        out.push_str(&format!("  + {r}\n"));
    }
    if !res.warnings.is_empty() {
        out.push_str("\nWarnings:\n");
        for w in &res.warnings {
            out.push_str(&format!("  ! {w}\n"));
        }
    }
    out
}
