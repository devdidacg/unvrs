use crate::executor::CommandSpec;
use crate::package::OperatingSystem;
use crate::resolver::{BackendClass, Resolution};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Install,
    Remove,
}

impl Operation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Operation::Install => "install",
            Operation::Remove => "remove",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlannedAction {
    pub requested: String,
    pub resolved: String,
    pub backend: String,
    pub class: BackendClass,
    pub source: String,
    pub command: Option<String>,
    pub requires_root: bool,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Plan {
    pub operation: Operation,
    pub actions: Vec<PlannedAction>,
    pub warnings: Vec<String>,
    pub requires_root: bool,
    /// No backend currently supports safe rollback; plans state this honestly.
    pub rollback_supported: bool,
    pub os: String,
    pub arch: String,
}

pub struct PlanBuilder<'a> {
    pub os: &'a OperatingSystem,
}

impl<'a> PlanBuilder<'a> {
    pub fn build(
        &self,
        operation: Operation,
        resolutions: &[Resolution],
        source_labels: &[String],
    ) -> Plan {
        let mut warnings: Vec<String> = Vec::new();

        let actions: Vec<PlannedAction> = resolutions
            .iter()
            .zip(source_labels.iter())
            .map(|(res, source)| {
                for w in &res.warnings {
                    if !warnings.contains(w) {
                        warnings.push(w.clone());
                    }
                }
                PlannedAction {
                    requested: res.requested.clone(),
                    resolved: res.resolved_name.clone(),
                    backend: res.backend_name.clone(),
                    class: res.class,
                    source: source.clone(),
                    command: None,
                    requires_root: false,
                    reasons: res.reasons.clone(),
                    warnings: res.warnings.clone(),
                }
            })
            .collect();

        // Requires_root and commands are attached by the dispatcher via
        // `attach_commands`, which owns the registry handles.
        Plan {
            operation,
            actions,
            warnings,
            requires_root: false,
            rollback_supported: false,
            os: self.os.label(),
            arch: self.os.arch.clone(),
        }
    }
}

/// Fill in each action's exact command + privilege requirement using the
/// matching backend's spec. Called by the dispatcher after `PlanBuilder::build`.
pub fn attach_commands<F>(plan: &mut Plan, mut spec_for: F)
where
    F: FnMut(&str, Operation, &str) -> Option<(CommandSpec, bool)>,
{
    for action in &mut plan.actions {
        if let Some((spec, needs_root)) =
            spec_for(&action.backend, plan.operation, &action.resolved)
        {
            action.command = Some(spec.display);
            action.requires_root = needs_root;
            if needs_root {
                plan.requires_root = true;
            }
        }
    }
}

/// Human-readable rendering of a plan (the `unvrs plan` output).
pub fn render_human(plan: &Plan) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "UNVRS {} PLAN\n",
        plan.operation.as_str().to_uppercase()
    ));
    out.push('\n');

    for (i, a) in plan.actions.iter().enumerate() {
        if plan.actions.len() > 1 {
            out.push_str(&format!("[{}/{}]\n", i + 1, plan.actions.len()));
        }
        out.push_str(&format!(
            "Package:  {} (resolved: {})\n",
            a.requested, a.resolved
        ));
        out.push_str(&format!("Backend:  {} ({})\n", a.backend, a.class));
        out.push_str(&format!("Source:   {}\n", a.source));
        out.push('\n');
        out.push_str("Reason:\n");
        for r in &a.reasons {
            out.push_str(&format!("  + {r}\n"));
        }
        out.push('\n');
        out.push_str("Command:\n");
        match &a.command {
            Some(cmd) => {
                if a.requires_root {
                    out.push_str(&format!("  sudo {cmd}\n"));
                } else {
                    out.push_str(&format!("  {cmd}\n"));
                }
            }
            None => out.push_str("  (command unavailable for this backend)\n"),
        }
        out.push('\n');
    }

    if !plan.warnings.is_empty() {
        out.push_str("Warnings:\n");
        for w in &plan.warnings {
            out.push_str(&format!("  ! {w}\n"));
        }
        out.push('\n');
    }

    if plan.requires_root {
        out.push_str("Privileges: root/sudo required\n");
    } else {
        out.push_str("Privileges: none required\n");
    }
    if !plan.rollback_supported {
        out.push_str("Rollback:   not supported by the selected backend(s)\n");
    }
    out.push_str(&format!("System:     {} ({})\n", plan.os, plan.arch));
    out.push('\n');
    out.push_str("No changes have been made.\n");
    out
}

/// Compact one-line preview used right before real execution.
pub fn render_exec_preview(plan: &Plan) -> String {
    let mut lines = Vec::new();
    for a in &plan.actions {
        match &a.command {
            Some(cmd) => {
                if a.requires_root {
                    lines.push(format!("sudo {cmd}"));
                } else {
                    lines.push(cmd.clone());
                }
            }
            None => lines.push(format!("# no command for {}", a.backend)),
        }
    }
    lines.join(" && ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::{OsFamily, PackageCandidate, PackageSource};
    use crate::resolver::Resolution;

    fn os() -> OperatingSystem {
        OperatingSystem {
            id: "ubuntu".into(),
            name: "Ubuntu".into(),
            version: Some("24.04".into()),
            family: OsFamily::Linux,
            arch: "x86_64".into(),
            id_like: vec!["debian".into()],
        }
    }

    fn resolution() -> Resolution {
        Resolution {
            requested: "firefox".into(),
            resolved_name: "firefox".into(),
            backend_name: "apt".into(),
            class: BackendClass::Native,
            candidate: PackageCandidate {
                name: "firefox".into(),
                version: None,
                source: PackageSource::System,
                backend: "apt".into(),
                architecture: None,
                description: None,
            },
            reasons: vec!["native package manager".into()],
            warnings: vec![],
            report: vec![],
        }
    }

    #[test]
    fn plan_contains_command_and_privileges() {
        let mut plan = PlanBuilder { os: &os() }.build(
            Operation::Install,
            &[resolution()],
            &["Ubuntu repository (apt)".into()],
        );
        attach_commands(&mut plan, |backend, op, pkg| {
            assert_eq!(backend, "apt");
            assert_eq!(op, Operation::Install);
            assert_eq!(pkg, "firefox");
            Some((CommandSpec::new("apt-get", ["install", "-y", pkg]), true))
        });

        assert!(plan.requires_root);
        assert!(!plan.rollback_supported);
        let action = &plan.actions[0];
        assert_eq!(
            action.command.as_deref(),
            Some("apt-get install -y firefox")
        );

        let human = render_human(&plan);
        assert!(human.contains("UNVRS INSTALL PLAN"));
        assert!(human.contains("sudo apt-get install -y firefox"));
        assert!(human.contains("No changes have been made."));
        assert!(human.contains("Root/sudo required") || human.contains("root/sudo required"));
    }

    #[test]
    fn json_plan_is_serializable() {
        let plan = PlanBuilder { os: &os() }.build(
            Operation::Install,
            &[resolution()],
            &["Ubuntu repository (apt)".into()],
        );
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("\"operation\":\"install\""));
        assert!(json.contains("\"rollback_supported\":false"));
    }

    #[test]
    fn exec_preview_prefixes_sudo() {
        let mut plan = PlanBuilder { os: &os() }.build(
            Operation::Install,
            &[resolution()],
            &["Ubuntu repository (apt)".into()],
        );
        attach_commands(&mut plan, |_, _, pkg| {
            Some((CommandSpec::new("apt-get", ["install", "-y", pkg]), true))
        });
        assert_eq!(
            render_exec_preview(&plan),
            "sudo apt-get install -y firefox"
        );
    }
}
