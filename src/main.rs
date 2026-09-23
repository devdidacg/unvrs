use clap::Parser;
use unvrs::cli::{Cli, Commands, HistoryAction, PlanOperation, ProfileAction};
use unvrs::config::Config;
use unvrs::dispatcher::{Dispatcher, SearchStatus};
use unvrs::error::{Result, UnvrsError};
use unvrs::executor;
use unvrs::history;
use unvrs::plan::{self, Operation, Plan};
use unvrs::profile;
use unvrs::resolver::{render_explain, ResolveOptions};
use unvrs::{context, doctor, ui};

fn main() {
    let cli = Cli::parse();
    context::init(cli.no_color, cli.json, cli.verbose);

    if let Err(e) = run(cli) {
        print_error(&e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let config = Config::load()?;
    let d = Dispatcher::new(config);

    match cli.command {
        Commands::Search { package, backend } => cmd_search(&d, &package, backend),
        Commands::Info { package } => cmd_info(&d, &package),
        Commands::Install {
            packages,
            dry,
            backend,
            cross_distro,
            force,
            container,
            explain,
        } => cmd_install_remove(
            &d,
            Operation::Install,
            packages,
            dry,
            backend,
            cross_distro || force,
            container,
            explain,
        ),
        Commands::Remove {
            packages,
            dry,
            backend,
            cross_distro,
            explain,
        } => cmd_install_remove(
            &d,
            Operation::Remove,
            packages,
            dry,
            backend,
            cross_distro,
            false,
            explain,
        ),
        Commands::Plan { operation } => match operation {
            PlanOperation::Install {
                packages,
                backend,
                cross_distro,
                container,
            } => cmd_plan_only(
                &d,
                Operation::Install,
                packages,
                backend,
                cross_distro,
                container,
            ),
            PlanOperation::Remove {
                packages,
                backend,
                cross_distro,
            } => cmd_plan_only(
                &d,
                Operation::Remove,
                packages,
                backend,
                cross_distro,
                false,
            ),
        },
        Commands::Apply {
            profile,
            dry,
            explain,
            backend,
        } => cmd_apply(&d, &profile, dry, explain, backend),
        Commands::Profile { action } => cmd_profile(&d, action),
        Commands::Update => cmd_update(&d),
        Commands::Upgrade => cmd_upgrade(&d),
        Commands::List => cmd_list(&d),
        Commands::Outdated => cmd_outdated(&d),
        Commands::History { action } => cmd_history(action),
        Commands::Clean => cmd_clean(&d),
        Commands::Doctor => cmd_doctor(),
    }
}

// ---------- shared helpers ----------

fn json_mode() -> bool {
    context::json()
}

fn print_error(e: &UnvrsError) {
    if json_mode() {
        let mut obj = serde_json::json!({ "error": e.to_string() });
        if let UnvrsError::UnknownBackend {
            suggestion: Some(s),
            ..
        } = e
        {
            obj["did_you_mean"] = serde_json::Value::String(s.clone());
        }
        if let Some(s) = e.suggestion() {
            obj["suggestion"] = serde_json::Value::String(s);
        }
        println!(
            "{}",
            serde_json::to_string(&obj).unwrap_or_else(|_| r#"{"error":"unknown"}"#.into())
        );
    } else {
        eprintln!("\n  {} {}", ui::icon_fail(), e);
        if let UnvrsError::UnknownBackend {
            suggestion: Some(s),
            ..
        } = e
        {
            eprintln!("  {} did you mean `{s}`?", ui::icon_info());
        }
        if let Some(s) = e.suggestion() {
            eprintln!("  {}", ui::dim(&s));
        }
    }
}

fn resolve_opts(
    backend: Option<String>,
    cross_distro: bool,
    container: bool,
    for_remove: bool,
    d: &Dispatcher,
) -> ResolveOptions {
    ResolveOptions {
        explicit_backend: backend,
        cross_distro,
        prefer_container: container,
        installed_packages: if for_remove {
            Some(d.installed_map())
        } else {
            None
        },
    }
}

fn print_explanations(d: &Dispatcher, resolutions: &[unvrs::resolver::Resolution]) {
    if json_mode() {
        return;
    }
    for r in resolutions {
        println!("{}", render_explain(d.os(), r));
        println!("{}", dim_bar());
    }
}

fn dim_bar() -> String {
    ui::dim("----------------------------------------------------------------")
}

fn print_plan_human(p: &Plan) {
    if json_mode() {
        return;
    }
    println!();
    print!("{}", plan::render_human(p));
}

fn print_plan_json(p: &Plan) {
    println!(
        "{}",
        serde_json::to_string_pretty(p).unwrap_or_else(|_| "{}".into())
    );
}

fn warn_privileges(p: &Plan) {
    if json_mode() {
        return;
    }
    if p.requires_root && !executor::is_root() && executor::is_sudo_available() {
        println!("  {} {}", ui::icon_warn(), ui::dim("may require sudo"));
    }
}

/// Execute a built plan, print results, exit non-zero on failure.
fn finish_execution(
    d: &Dispatcher,
    p: &Plan,
    resolutions: &[unvrs::resolver::Resolution],
    explain: bool,
) -> Result<()> {
    if explain {
        print_explanations(d, resolutions);
    }

    if p.operation == Operation::Install || p.operation == Operation::Remove {
        warn_privileges(p);
    }

    if json_mode() {
        // dry-run path handled by caller; here we always execute
    } else {
        println!();
        println!(
            "  {} {}",
            ui::icon_info(),
            ui::dim(&format!("Executing: {}", plan::render_exec_preview(p)))
        );
        println!();
    }

    let spinner = ui::Spinner::new(match p.operation {
        Operation::Install => "Installing...",
        Operation::Remove => "Removing...",
    });
    let outcomes = d.execute_plan(p)?;
    let all_ok = outcomes.len() == p.actions.len() && outcomes.iter().all(|o| o.result.success);

    // Always stop the spinner before printing results (no-op in JSON mode).
    if all_ok {
        spinner.stop_with("Done");
    } else {
        spinner.stop_fail("Failed");
    }

    if json_mode() {
        let actions: Vec<_> = outcomes
            .iter()
            .map(|o| {
                serde_json::json!({
                    "requested": o.requested,
                    "resolved": o.resolved,
                    "backend": o.backend,
                    "command": o.command,
                    "success": o.result.success,
                    "message": o.result.message,
                    "exit_code": o.result.exit_code,
                    "history_id": o.history_id,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "success": all_ok,
                "operation": p.operation.as_str(),
                "actions": actions,
            }))
            .unwrap_or_default()
        );
    } else {
        println!();
        for o in &outcomes {
            let icon = if o.result.success {
                ui::icon_ok()
            } else {
                ui::icon_fail()
            };
            println!("  {} {}", icon, o.result.message);
            if !o.result.success {
                for line in o.result.message.lines().skip(1) {
                    println!("     {}", line);
                }
            }
        }
        if all_ok {
            println!(
                "  {} recorded as transaction #{}",
                ui::icon_info(),
                outcomes.last().map(|o| o.history_id).unwrap_or(0)
            );
        }
        println!();
    }

    if !all_ok {
        std::process::exit(1);
    }
    Ok(())
}

// ---------- commands ----------

fn cmd_search(d: &Dispatcher, package: &str, backend: Option<String>) -> Result<()> {
    let spinner = ui::Spinner::new(&format!("Searching for {package}..."));

    let results = if let Some(ref backend_name) = backend {
        let candidates = d.search_in_backend(package, backend_name)?;
        if candidates.is_empty() {
            vec![(backend_name.clone(), SearchStatus::NotFound)]
        } else {
            vec![(backend_name.clone(), SearchStatus::Found(candidates))]
        }
    } else {
        d.search_with_status(package)
    };

    spinner.stop_with(&format!("Searched {package}"));

    if json_mode() {
        let mut all = Vec::new();
        for (_, status) in &results {
            if let SearchStatus::Found(candidates) = status {
                all.extend(candidates.iter().cloned());
            }
        }
        println!("{}", serde_json::to_string_pretty(&all).unwrap_or_default());
        return Ok(());
    }

    println!();
    for (name, status) in &results {
        match status {
            SearchStatus::Found(_) => println!("  {} {}", ui::icon_ok(), name),
            SearchStatus::NotFound => println!("  {} {}", ui::icon_fail(), name),
            SearchStatus::NotImplemented => {
                println!("  {} {} {}", ui::icon_warn(), name, ui::dim("(stub)"))
            }
            SearchStatus::Error => println!("  {} {}", ui::icon_fail(), name),
        }
    }

    let mut found_any = false;
    for (name, status) in &results {
        if let SearchStatus::Found(candidates) = status {
            if !found_any {
                println!("\n  {}", ui::bold("Results:"));
                found_any = true;
            }
            for c in candidates {
                let ver = c
                    .version
                    .as_deref()
                    .map(|v| format!(" {v}"))
                    .unwrap_or_default();
                println!(
                    "  {} {}{} {}",
                    ui::icon_info(),
                    c.name,
                    ver,
                    ui::dim(&format!("({name})"))
                );
                if let Some(desc) = &c.description {
                    println!("    {}", ui::dim(desc));
                }
            }
        }
    }

    if !found_any {
        println!("\n  {} No packages found for '{package}'.", ui::icon_fail());
    }
    println!();
    Ok(())
}

fn cmd_info(d: &Dispatcher, package: &str) -> Result<()> {
    let spinner = ui::Spinner::new(&format!("Looking up {package}..."));
    let info = d.info(package)?;
    spinner.stop_with(&format!("Found {package}"));

    if json_mode() {
        match info {
            Some(info) => println!(
                "{}",
                serde_json::to_string_pretty(&info).unwrap_or_default()
            ),
            None => println!("null"),
        }
        return Ok(());
    }

    match info {
        Some(info) => {
            println!();
            println!(
                "  {} {}",
                ui::bold(&info.name),
                info.version.unwrap_or_default()
            );
            if let Some(desc) = &info.description {
                println!("  {}", ui::dim(desc));
            }
            println!();
            if let Some(a) = &info.architecture {
                println!("  {} arch     {a}", ui::icon_info());
            }
            if let Some(m) = &info.maintainer {
                println!("  {} maint    {m}", ui::icon_info());
            }
            if let Some(h) = &info.homepage {
                println!("  {} home     {h}", ui::icon_info());
            }
            if !info.dependencies.is_empty() {
                println!(
                    "  {} deps     {}",
                    ui::icon_info(),
                    info.dependencies.join(", ")
                );
            }
            println!("  {} backend  {}", ui::icon_info(), ui::dim(&info.backend));
        }
        None => {
            println!(
                "\n  {} No information found for '{package}'.",
                ui::icon_fail()
            );
        }
    }
    println!();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_install_remove(
    d: &Dispatcher,
    operation: Operation,
    packages: Vec<String>,
    dry: bool,
    backend: Option<String>,
    cross_distro: bool,
    container: bool,
    explain: bool,
) -> Result<()> {
    let for_remove = operation == Operation::Remove;
    let opts = resolve_opts(backend, cross_distro, container, for_remove, d);

    let spinner = ui::Spinner::new("Resolving...");
    let (p, resolutions) = d.build_plan(operation, &packages, &opts)?;
    spinner.stop_with("Resolved");

    if explain {
        print_explanations(d, &resolutions);
    }

    if dry {
        if json_mode() {
            print_plan_json(&p);
        } else {
            print_plan_human(&p);
        }
        return Ok(());
    }

    finish_execution(d, &p, &resolutions, false)
}

fn cmd_plan_only(
    d: &Dispatcher,
    operation: Operation,
    packages: Vec<String>,
    backend: Option<String>,
    cross_distro: bool,
    container: bool,
) -> Result<()> {
    let for_remove = operation == Operation::Remove;
    let opts = resolve_opts(backend, cross_distro, container, for_remove, d);
    let (p, resolutions) = d.build_plan(operation, &packages, &opts)?;

    if json_mode() {
        print_plan_json(&p);
    } else {
        print_plan_human(&p);
    }
    let _ = resolutions;
    Ok(())
}

fn cmd_apply(
    d: &Dispatcher,
    name: &str,
    dry: bool,
    explain: bool,
    backend: Option<String>,
) -> Result<()> {
    let prof = profile::load(name)?;
    let backend = backend.or_else(|| prof.backend.clone());
    let opts = resolve_opts(backend, false, false, false, d);

    let spinner = ui::Spinner::new(&format!("Resolving profile `{name}`..."));
    let (p, resolutions) = d.build_plan(Operation::Install, &prof.packages, &opts)?;
    spinner.stop_with("Resolved");

    if explain {
        print_explanations(d, &resolutions);
    }

    if dry {
        if json_mode() {
            print_plan_json(&p);
        } else {
            print_plan_human(&p);
        }
        return Ok(());
    }

    finish_execution(d, &p, &resolutions, false)
}

fn cmd_profile(d: &Dispatcher, action: ProfileAction) -> Result<()> {
    match action {
        ProfileAction::Create { name, packages } => {
            let p = profile::create(&name, packages)?;
            if json_mode() {
                println!("{}", serde_json::to_string_pretty(&p).unwrap_or_default());
            } else {
                println!(
                    "  {} profile `{}` created ({} packages)",
                    ui::icon_ok(),
                    p.name,
                    p.packages.len()
                );
            }
            Ok(())
        }
        ProfileAction::List => {
            let profiles = profile::list()?;
            if json_mode() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&profiles).unwrap_or_default()
                );
            } else {
                println!();
                if profiles.is_empty() {
                    println!("  {} no profiles yet", ui::icon_info());
                }
                for p in profiles {
                    println!(
                        "  {} {} {}",
                        ui::icon_info(),
                        ui::bold(&p.name),
                        ui::dim(&format!("{} packages", p.packages.len()))
                    );
                }
                println!();
            }
            Ok(())
        }
        ProfileAction::Show { name } => {
            let p = profile::load(&name)?;
            if json_mode() {
                println!("{}", serde_json::to_string_pretty(&p).unwrap_or_default());
            } else {
                println!();
                println!("  {}", ui::bold(&p.name));
                for pkg in &p.packages {
                    println!("    {} {}", ui::icon_info(), pkg);
                }
                if let Some(b) = &p.backend {
                    println!("  {} backend {b}", ui::icon_info());
                }
                println!();
            }
            Ok(())
        }
        ProfileAction::Plan { name } => cmd_apply(d, &name, true, false, None),
        ProfileAction::Apply { name } => cmd_apply(d, &name, false, false, None),
    }
}

fn cmd_update(d: &Dispatcher) -> Result<()> {
    if !json_mode() {
        println!();
    }
    let spinner = ui::Spinner::new("Updating package lists...");
    let results = d.update()?;
    spinner.stop_with("Package lists updated");
    print_results(&results, false)
}

fn cmd_upgrade(d: &Dispatcher) -> Result<()> {
    if !json_mode() {
        println!();
    }
    let spinner = ui::Spinner::new("Upgrading packages...");
    let results = d.upgrade()?;
    spinner.stop_with("Upgrade complete");
    print_results(&results, false)
}

fn cmd_clean(d: &Dispatcher) -> Result<()> {
    if !json_mode() {
        println!();
    }
    let spinner = ui::Spinner::new("Cleaning package caches...");
    let results = d.clean()?;
    spinner.stop_with("Cache cleaned");
    print_results(&results, true)
}

fn print_results(results: &[unvrs::package::InstallationResult], warn_on_fail: bool) -> Result<()> {
    if json_mode() {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "success": r.success,
                            "backend": r.backend,
                            "message": r.message,
                            "exit_code": r.exit_code,
                        })
                    })
                    .collect::<Vec<_>>()
            )
            .unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    for r in results {
        if r.success {
            println!("  {} {}", ui::icon_ok(), r.message);
        } else if warn_on_fail {
            println!("  {} {}", ui::icon_warn(), r.message);
        } else {
            println!("  {} {}", ui::icon_fail(), r.message);
        }
    }
    println!();
    Ok(())
}

fn cmd_list(d: &Dispatcher) -> Result<()> {
    let spinner = ui::Spinner::new("Listing installed packages...");
    let packages = d.list_installed()?;
    spinner.stop_with(&format!("Found {} packages", packages.len()));

    if json_mode() {
        println!(
            "{}",
            serde_json::to_string_pretty(&packages).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    if packages.is_empty() {
        println!(
            "  {} {}",
            ui::icon_warn(),
            ui::dim("no packages found via available backends")
        );
    } else {
        for p in &packages {
            println!(
                "  {} {} {} {}",
                ui::icon_info(),
                ui::bold(&p.name),
                ui::dim(&p.version),
                ui::dim(&format!("({})", p.backend))
            );
        }
    }
    println!();
    Ok(())
}

fn cmd_outdated(d: &Dispatcher) -> Result<()> {
    let spinner = ui::Spinner::new("Checking for outdated packages...");
    let packages = d.outdated()?;
    spinner.stop_with(&format!("Found {} outdated", packages.len()));

    if json_mode() {
        println!(
            "{}",
            serde_json::to_string_pretty(&packages).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    if packages.is_empty() {
        println!("  {} All packages are up to date.", ui::icon_ok());
    } else {
        for p in &packages {
            println!(
                "  {} {} {} -> {} {}",
                ui::icon_warn(),
                ui::bold(&p.name),
                ui::dim(&p.current_version),
                ui::green(&p.latest_version),
                ui::dim(&format!("({})", p.backend))
            );
        }
    }
    println!();
    Ok(())
}

fn cmd_history(action: Option<HistoryAction>) -> Result<()> {
    match action {
        Some(HistoryAction::Show { id }) => {
            let e = history::get(id).ok_or_else(|| {
                UnvrsError::ConfigurationError(format!(
                    "no transaction with id {id} (run `unvrs history` to list)"
                ))
            })?;
            if json_mode() {
                println!("{}", serde_json::to_string_pretty(&e).unwrap_or_default());
            } else {
                println!();
                println!("  {}", ui::bold(&format!("Transaction #{}", e.id)));
                println!(
                    "  {} {} {} via {}",
                    if e.success {
                        ui::icon_ok()
                    } else {
                        ui::icon_fail()
                    },
                    e.action,
                    ui::bold(&e.package),
                    ui::dim(&e.backend)
                );
                let date = chrono::DateTime::from_timestamp(e.timestamp as i64, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "unknown".into());
                println!("  {} {date}", ui::icon_info());
                if !e.command.is_empty() {
                    println!("  {} {}", ui::icon_info(), e.command);
                }
                if let Some(code) = e.exit_code {
                    println!("  {} exit code {code}", ui::icon_info());
                }
                for w in &e.warnings {
                    println!("  {} {w}", ui::icon_warn());
                }
                if !e.rollback_supported {
                    println!(
                        "  {} rollback not supported for this backend",
                        ui::icon_info()
                    );
                }
                println!();
            }
            Ok(())
        }
        None => {
            let entries = history::load();
            if json_mode() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&entries).unwrap_or_default()
                );
                return Ok(());
            }

            println!();
            println!("  {}", ui::bold("Transaction History"));
            println!();
            if entries.is_empty() {
                println!("  {} No history found.", ui::icon_info());
            } else {
                for e in entries.iter().rev().take(20) {
                    let date = chrono::DateTime::from_timestamp(e.timestamp as i64, 0)
                        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "unknown".into());
                    let icon = if e.success {
                        ui::icon_ok()
                    } else {
                        ui::icon_fail()
                    };
                    println!(
                        "  {} #{:<4} {} {} {} {}",
                        icon,
                        e.id,
                        ui::dim(&date),
                        ui::dim(&e.action),
                        ui::bold(&e.package),
                        ui::dim(&format!("({})", e.backend))
                    );
                }
            }
            println!();
            Ok(())
        }
    }
}

fn cmd_doctor() -> Result<()> {
    let report = doctor::run(false);

    if json_mode() {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("  {}", ui::bold("unvrs doctor"));
    println!();

    println!("  {}", ui::bold("System"));
    println!(
        "  {} {} ({}), {}",
        ui::icon_ok(),
        report.system.os,
        report.system.family,
        report.system.arch
    );

    println!();
    println!("  {}", ui::bold("Backends"));
    for b in &report.backends {
        let line = if b.available && b.compatible {
            let mut s = format!("{:12} {}", b.name, ui::dim("ready"));
            if let Some(v) = &b.version {
                s.push_str(&format!(" {}", ui::dim(v)));
            }
            Some((ui::icon_ok(), s))
        } else if b.available && !b.compatible {
            Some((
                ui::icon_warn(),
                format!("{:12} {}", b.name, ui::dim("wrong OS")),
            ))
        } else if b.capabilities.can_search {
            Some((
                ui::icon_warn(),
                format!("{:12} {}", b.name, ui::dim("not installed")),
            ))
        } else {
            Some((
                ui::icon_fail(),
                format!("{:12} {}", b.name, ui::dim("stub")),
            ))
        };
        if let Some((icon, s)) = line {
            println!("  {icon} {s}");
        }
    }

    println!();
    println!("  {}", ui::bold("Privileges"));
    if report.privileges.root {
        println!("  {} {}", ui::icon_ok(), ui::dim("root"));
    } else if report.privileges.sudo_available {
        println!("  {} {}", ui::icon_ok(), ui::dim("sudo available"));
    } else {
        println!("  {} {}", ui::icon_warn(), ui::dim("no sudo found"));
    }

    println!();
    println!("  {}", ui::bold("Config"));
    if report.config.exists && report.config.valid {
        println!("  {} {}", ui::icon_ok(), report.config.path);
    } else if report.config.exists && !report.config.valid {
        println!(
            "  {} {} {}",
            ui::icon_fail(),
            report.config.path,
            report.config.error.unwrap_or_default()
        );
    } else {
        println!(
            "  {} {}",
            ui::icon_warn(),
            ui::dim("no config file (using defaults)")
        );
    }

    println!();
    println!("  {}", ui::bold("Status"));
    println!("  {} {}", ui::icon_ok(), report.status);
    for issue in &report.issues {
        println!("  {} {issue}", ui::icon_warn());
    }
    println!();
    Ok(())
}
