use crate::backends;
use crate::config::Config;
use crate::executor;
use crate::os;
use crate::package::OperatingSystem;

#[derive(Debug, serde::Serialize)]
pub struct BackendReport {
    pub name: String,
    pub available: bool,
    pub native: bool,
    pub universal: bool,
    pub compatible: bool,
    pub requires_root: bool,
    pub version: Option<String>,
    pub capabilities: crate::package::BackendCapabilities,
}

#[derive(Debug, serde::Serialize)]
pub struct DoctorReport {
    pub system: SystemReport,
    pub backends: Vec<BackendReport>,
    pub config: ConfigReport,
    pub privileges: PrivilegeReport,
    pub status: String,
    pub issues: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SystemReport {
    pub os: String,
    pub id: String,
    pub family: String,
    pub arch: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ConfigReport {
    pub path: String,
    pub exists: bool,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct PrivilegeReport {
    pub root: bool,
    pub sudo_available: bool,
}

/// Run all diagnostics. Performs only read-only/version probes.
pub fn run(skip_versions: bool) -> DoctorReport {
    let os: OperatingSystem = os::detect();
    let config_path = Config::config_path();

    let (config_valid, config_error) = match Config::load_from_path(&config_path) {
        Ok(_) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };
    let mut issues = Vec::new();

    let backends: Vec<BackendReport> = backends::all_backends()
        .iter()
        .map(|b| {
            let available = b.is_available();
            let version = if skip_versions || !available {
                None
            } else {
                b.version_probe()
                    .and_then(|spec| backends::probe_version(&spec))
            };
            BackendReport {
                name: b.name().to_string(),
                available,
                native: b.is_native_for(&os),
                universal: b.is_universal(),
                compatible: b.is_compatible(&os) || b.is_universal(),
                requires_root: b.requires_root(),
                version,
                capabilities: b.capabilities(),
            }
        })
        .collect();

    if !config_valid {
        issues.push(config_error.clone().unwrap_or_default());
    }
    if !backends.iter().any(|b| b.available) {
        issues.push("no package manager backends are installed".into());
    }

    let root = executor::is_root();
    let sudo_available = executor::is_sudo_available();
    if !root && !sudo_available && backends.iter().any(|b| b.available && b.requires_root) {
        issues.push("some backends need root but neither root nor sudo is available".to_string());
    }

    let status = if issues.is_empty() {
        "Ready".to_string()
    } else {
        format!("{} issue(s) found", issues.len())
    };

    DoctorReport {
        system: SystemReport {
            os: os.label(),
            id: os.id,
            family: os.family.to_string(),
            arch: os.arch,
        },
        backends,
        config: ConfigReport {
            path: config_path.display().to_string(),
            exists: config_path.exists(),
            valid: config_valid,
            error: config_error,
        },
        privileges: PrivilegeReport {
            root,
            sudo_available,
        },
        status,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_runs_and_reports_system() {
        let r = run(true);
        assert!(!r.system.os.is_empty());
        assert!(!r.system.arch.is_empty());
        assert_eq!(r.backends.len(), backends::all_backends().len());
        // Doctor itself must never panic regardless of host setup.
        let _json = serde_json::to_string(&r).unwrap();
    }

    #[test]
    fn doctor_json_shape_is_stable() {
        let r = run(true);
        let json = serde_json::to_value(&r).unwrap();
        for key in [
            "system",
            "backends",
            "config",
            "privileges",
            "status",
            "issues",
        ] {
            assert!(json.get(key).is_some(), "missing key {key}");
        }
        let b0 = &json["backends"][0];
        for key in ["name", "available", "capabilities", "version"] {
            assert!(b0.get(key).is_some(), "missing backend key {key}");
        }
        assert!(b0["capabilities"].get("rollback").is_some());
        assert!(b0["capabilities"].get("dry_run").is_some());
    }
}
