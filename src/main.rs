use clap::Parser;
use unvrs::cli::{Cli, Commands};
use unvrs::config::Config;
use unvrs::executor;
use unvrs::resolver::{Resolver, SearchStatus};
use unvrs::ui;

static mut NO_COLOR: bool = false;
static mut JSON_MODE: bool = false;

fn main() {
    let cli = Cli::parse();

    unsafe {
        NO_COLOR = cli.no_color;
        JSON_MODE = cli.json;
    }

    let config = Config::load();
    let resolver = Resolver::new(config);

    let result = match cli.command {
        Commands::Search { package } => cmd_search(&resolver, &package),
        Commands::Info { package } => cmd_info(&resolver, &package),
        Commands::Install { package } => cmd_install(&resolver, &package),
        Commands::Remove { package } => cmd_remove(&resolver, &package),
        Commands::Update => cmd_update(&resolver),
        Commands::Upgrade => cmd_upgrade(&resolver),
        Commands::List => cmd_list(&resolver),
        Commands::Outdated => cmd_outdated(&resolver),
        Commands::Doctor => cmd_doctor(&resolver),
    };

    if let Err(e) = result {
        if json_mode() {
            println!("{{\"error\":\"{}\"}}", e);
        } else {
            eprintln!("\n  {} {e}", ui::icon_fail());
        }
        std::process::exit(1);
    }
}

fn no_color() -> bool {
    unsafe { NO_COLOR }
}

fn json_mode() -> bool {
    unsafe { JSON_MODE }
}

fn icon_ok() -> &'static str {
    if no_color() {
        "OK"
    } else {
        ui::icon_ok()
    }
}

fn icon_fail() -> &'static str {
    if no_color() {
        "FAIL"
    } else {
        ui::icon_fail()
    }
}

fn icon_warn() -> &'static str {
    if no_color() {
        "WARN"
    } else {
        ui::icon_warn()
    }
}

fn icon_info() -> &'static str {
    if no_color() {
        "*"
    } else {
        ui::icon_info()
    }
}

fn print_status_line(backend_name: &str, status: &SearchStatus) {
    if json_mode() {
        return;
    }
    match status {
        SearchStatus::Found(_) => println!("  {} {}", icon_ok(), backend_name),
        SearchStatus::NotFound => println!("  {} {}", icon_fail(), backend_name),
        SearchStatus::NotImplemented => {
            let msg = if no_color() {
                "(stub)".to_string()
            } else {
                ui::dim("(stub)")
            };
            println!("  {} {} {}", icon_warn(), backend_name, msg)
        }
        SearchStatus::Error => println!("  {} {}", icon_fail(), backend_name),
    }
}

fn cmd_search(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    let spinner = ui::Spinner::new(&format!("Searching for {package}..."));
    let results = resolver.search_with_status(package);
    spinner.stop_with(&format!("Searched {package}"));

    if json_mode() {
        let mut all_candidates = Vec::new();
        for (_name, status) in &results {
            if let SearchStatus::Found(candidates) = status {
                for c in candidates {
                    all_candidates.push(c.clone());
                }
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&all_candidates).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    for (name, status) in &results {
        print_status_line(name, status);
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
                    icon_info(),
                    c.name,
                    ver,
                    if no_color() {
                        format!("({name})")
                    } else {
                        ui::dim(&format!("({name})"))
                    }
                );
                if let Some(desc) = &c.description {
                    println!(
                        "    {}",
                        if no_color() {
                            desc.clone()
                        } else {
                            ui::dim(desc)
                        }
                    );
                }
            }
        }
    }

    if !found_any {
        println!("\n  {} No packages found for '{package}'.", icon_fail());
    }
    println!();
    Ok(())
}

fn cmd_info(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    let spinner = ui::Spinner::new(&format!("Looking up {package}..."));
    let info = resolver.info(package)?;
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
            if let Some(d) = &info.description {
                println!("  {}", if no_color() { d.clone() } else { ui::dim(d) });
            }
            println!();
            if let Some(a) = &info.architecture {
                println!("  {} arch     {a}", icon_info());
            }
            if let Some(m) = &info.maintainer {
                println!("  {} maint    {m}", icon_info());
            }
            if let Some(h) = &info.homepage {
                println!("  {} home     {h}", icon_info());
            }
            if !info.dependencies.is_empty() {
                println!(
                    "  {} deps     {}",
                    icon_info(),
                    info.dependencies.join(", ")
                );
            }
            println!(
                "  {} backend  {}",
                icon_info(),
                if no_color() {
                    info.backend.clone()
                } else {
                    ui::dim(&info.backend)
                }
            );
        }
        None => {
            println!("\n  {} No information found for '{package}'.", icon_fail());
        }
    }
    println!();
    Ok(())
}

fn cmd_install(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    if !json_mode() {
        println!();
        if !executor::is_root() && executor::is_sudo_available() {
            println!(
                "  {} {}",
                icon_warn(),
                if no_color() {
                    "may require sudo".into()
                } else {
                    ui::dim("may require sudo")
                }
            );
        }
    }

    let spinner = ui::Spinner::new(&format!("Searching for {package}..."));
    let results = resolver.search_with_status(package);
    spinner.stop_with(&format!("Searched {package}"));

    if !json_mode() {
        println!();
        for (name, status) in &results {
            print_status_line(name, status);
        }

        let os = resolver.os();
        let mut incompatible_warned = false;
        for (name, status) in &results {
            if let SearchStatus::Found(_) = status {
                if let Some(backend) = resolver.registry().find_by_name(name) {
                    if backend.is_available() && !backend.is_compatible(os) {
                        if !incompatible_warned {
                            println!(
                                "\n  {} {}",
                                icon_warn(),
                                if no_color() {
                                    "Some backends are not native to your OS:".into()
                                } else {
                                    ui::dim("Some backends are not native to your OS:")
                                }
                            );
                            incompatible_warned = true;
                        }
                        println!(
                            "    {} {} {}",
                            icon_warn(),
                            name,
                            if no_color() {
                                format!("(not native to {})", os.family)
                            } else {
                                ui::dim(&format!("(not native to {})", os.family))
                            }
                        );
                    }
                }
            }
        }
        if incompatible_warned {
            println!();
        }
    }

    let spinner2 = ui::Spinner::new("Installing...");
    let result = resolver.install(package)?;

    if json_mode() {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "success": result.success,
                "backend": result.backend,
                "package": result.package,
                "message": result.message,
            }))
            .unwrap_or_default()
        );
    } else if result.success {
        spinner2.stop_with(&format!("{} {}", icon_ok(), result.message));
    } else {
        spinner2.stop_fail(&format!("{} {}", icon_fail(), result.message));
    }

    if !json_mode() {
        println!();
    }
    Ok(())
}

fn cmd_remove(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    if !json_mode() {
        println!();
    }
    let spinner = ui::Spinner::new(&format!("Removing {package}..."));
    let result = resolver.remove(package)?;

    if json_mode() {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "success": result.success,
                "backend": result.backend,
                "package": result.package,
                "message": result.message,
            }))
            .unwrap_or_default()
        );
    } else if result.success {
        spinner.stop_with(&format!("{} {}", icon_ok(), result.message));
    } else {
        spinner.stop_fail(&format!("{} {}", icon_fail(), result.message));
    }

    if !json_mode() {
        println!();
    }
    Ok(())
}

fn cmd_update(resolver: &Resolver) -> unvrs::error::Result<()> {
    if !json_mode() {
        println!();
    }
    let spinner = ui::Spinner::new("Updating package lists...");
    let results = resolver.update()?;
    spinner.stop_with("Package lists updated");

    if json_mode() {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &results
                    .iter()
                    .map(|r| serde_json::json!({
                        "success": r.success,
                        "backend": r.backend,
                        "message": r.message,
                    }))
                    .collect::<Vec<_>>()
            )
            .unwrap_or_default()
        );
    } else {
        println!();
        for r in &results {
            if r.success {
                println!("  {} {}", icon_ok(), r.message);
            } else {
                println!("  {} {}", icon_fail(), r.message);
            }
        }
        println!();
    }
    Ok(())
}

fn cmd_upgrade(resolver: &Resolver) -> unvrs::error::Result<()> {
    if !json_mode() {
        println!();
    }
    let spinner = ui::Spinner::new("Upgrading packages...");
    let results = resolver.upgrade()?;
    spinner.stop_with("Upgrade complete");

    if json_mode() {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &results
                    .iter()
                    .map(|r| serde_json::json!({
                        "success": r.success,
                        "backend": r.backend,
                        "message": r.message,
                    }))
                    .collect::<Vec<_>>()
            )
            .unwrap_or_default()
        );
    } else {
        println!();
        for r in &results {
            if r.success {
                println!("  {} {}", icon_ok(), r.message);
            } else {
                println!("  {} {}", icon_fail(), r.message);
            }
        }
        println!();
    }
    Ok(())
}

fn cmd_list(resolver: &Resolver) -> unvrs::error::Result<()> {
    let spinner = ui::Spinner::new("Listing installed packages...");
    let packages = resolver.list_installed()?;
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
            icon_warn(),
            if no_color() {
                "no packages found via available backends".into()
            } else {
                ui::dim("no packages found via available backends")
            }
        );
    } else {
        for p in &packages {
            println!(
                "  {} {} {} {}",
                icon_info(),
                ui::bold(&p.name),
                if no_color() {
                    p.version.clone()
                } else {
                    ui::dim(&p.version)
                },
                if no_color() {
                    format!("({})", p.backend)
                } else {
                    ui::dim(&format!("({})", p.backend))
                }
            );
        }
    }
    println!();
    Ok(())
}

fn cmd_outdated(resolver: &Resolver) -> unvrs::error::Result<()> {
    let spinner = ui::Spinner::new("Checking for outdated packages...");
    let packages = resolver.outdated()?;
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
        println!("  {} All packages are up to date.", icon_ok());
    } else {
        for p in &packages {
            println!(
                "  {} {} {} -> {} {}",
                icon_warn(),
                ui::bold(&p.name),
                if no_color() {
                    p.current_version.clone()
                } else {
                    ui::dim(&p.current_version)
                },
                ui::green(&p.latest_version),
                if no_color() {
                    format!("({})", p.backend)
                } else {
                    ui::dim(&format!("({})", p.backend))
                }
            );
        }
    }
    println!();
    Ok(())
}

fn cmd_doctor(resolver: &Resolver) -> unvrs::error::Result<()> {
    let os = resolver.os();

    if json_mode() {
        let backends: Vec<_> = resolver
            .registry()
            .backends()
            .iter()
            .map(|b| {
                serde_json::json!({
                    "name": b.name(),
                    "available": b.is_available(),
                    "compatible": b.is_compatible(os),
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "os": {
                    "name": os.name,
                    "family": os.family.to_string(),
                },
                "backends": backends,
                "root": executor::is_root(),
                "sudo": executor::is_sudo_available(),
                "config": unvrs::config::Config::config_path().exists(),
            }))
            .unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("  {}", ui::bold("unvrs doctor"));
    println!();

    println!("  {}", ui::bold("System"));
    println!("  {} os       {} ({})", icon_ok(), os.name, os.family);

    println!();
    println!("  {}", ui::bold("Backends"));
    for backend in resolver.registry().backends() {
        let compat = backend.is_compatible(os);
        let avail = backend.is_available();
        let has_caps = backend.capabilities().can_search;

        if avail && compat {
            println!(
                "  {} {:10} {}",
                icon_ok(),
                backend.name(),
                if no_color() {
                    "ready".into()
                } else {
                    ui::dim("ready")
                }
            );
        } else if avail && !compat {
            println!(
                "  {} {:10} {}",
                icon_warn(),
                backend.name(),
                if no_color() {
                    "wrong OS".into()
                } else {
                    ui::dim("wrong OS")
                }
            );
        } else if has_caps {
            println!(
                "  {} {:10} {}",
                icon_warn(),
                backend.name(),
                if no_color() {
                    "not installed".into()
                } else {
                    ui::dim("not installed")
                }
            );
        } else {
            println!(
                "  {} {:10} {}",
                icon_fail(),
                backend.name(),
                if no_color() {
                    "stub".into()
                } else {
                    ui::dim("stub")
                }
            );
        }
    }

    println!();
    println!("  {}", ui::bold("Privileges"));
    if executor::is_root() {
        println!(
            "  {} {}",
            icon_ok(),
            if no_color() {
                "root".into()
            } else {
                ui::dim("root")
            }
        );
    } else if executor::is_sudo_available() {
        println!(
            "  {} {}",
            icon_ok(),
            if no_color() {
                "sudo available".into()
            } else {
                ui::dim("sudo available")
            }
        );
    } else {
        println!(
            "  {} {}",
            icon_warn(),
            if no_color() {
                "no sudo found".into()
            } else {
                ui::dim("no sudo found")
            }
        );
    }

    println!();
    println!("  {}", ui::bold("Config"));
    let config_path = unvrs::config::Config::config_path();
    if config_path.exists() {
        println!("  {} {}", icon_ok(), config_path.display());
    } else {
        println!(
            "  {} {}",
            icon_warn(),
            if no_color() {
                "no config file (using defaults)".into()
            } else {
                ui::dim("no config file (using defaults)")
            }
        );
    }

    println!();
    Ok(())
}
