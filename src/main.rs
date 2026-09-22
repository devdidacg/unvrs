use clap::Parser;
use unvrs::cli::{Cli, Commands};
use unvrs::config::Config;
use unvrs::executor;
use unvrs::resolver::{Resolver, SearchStatus};
use unvrs::ui;

fn main() {
    let cli = Cli::parse();
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
        Commands::Doctor => cmd_doctor(&resolver),
    };

    if let Err(e) = result {
        eprintln!("\n  {} {e}", ui::icon_fail());
        std::process::exit(1);
    }
}

fn print_status_line(backend_name: &str, status: &SearchStatus) {
    match status {
        SearchStatus::Found(_) => println!("  {} {}", ui::icon_ok(), backend_name),
        SearchStatus::NotFound => println!("  {} {}", ui::icon_fail(), backend_name),
        SearchStatus::NotImplemented => {
            println!(
                "  {} {} {}",
                ui::icon_warn(),
                backend_name,
                ui::dim("(not implemented)")
            )
        }
        SearchStatus::Error => println!("  {} {}", ui::icon_fail(), backend_name),
    }
}

fn cmd_search(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    let spinner = ui::Spinner::new(&format!("Searching for {package}..."));
    let results = resolver.search_with_status(package);
    spinner.stop_with(&format!("Searched {package}"));

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

fn cmd_info(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    let spinner = ui::Spinner::new(&format!("Looking up {package}..."));
    let info = resolver.info(package)?;
    spinner.stop_with(&format!("Found {package}"));

    match info {
        Some(info) => {
            println!();
            println!(
                "  {} {}",
                ui::bold(&info.name),
                info.version.unwrap_or_default()
            );
            if let Some(d) = &info.description {
                println!("  {}", ui::dim(d));
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

fn cmd_install(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    println!();
    if !executor::is_root() && executor::is_sudo_available() {
        println!("  {} {}", ui::icon_warn(), ui::dim("may require sudo"));
    }

    let spinner = ui::Spinner::new(&format!("Searching for {package}..."));
    let results = resolver.search_with_status(package);
    spinner.stop_with(&format!("Searched {package}"));

    println!();
    for (name, status) in &results {
        print_status_line(name, status);
    }

    // Check OS compatibility
    let os = resolver.os();
    let mut incompatible_warned = false;
    for (name, status) in &results {
        if let SearchStatus::Found(_) = status {
            if let Some(backend) = resolver.registry().find_by_name(name) {
                if backend.is_available() && !backend.is_compatible(os) {
                    if !incompatible_warned {
                        println!(
                            "\n  {} {}",
                            ui::icon_warn(),
                            ui::dim("Some backends are not native to your OS:")
                        );
                        incompatible_warned = true;
                    }
                    println!(
                        "    {} {} {}",
                        ui::icon_warn(),
                        name,
                        ui::dim(&format!("(not native to {})", os.family))
                    );
                }
            }
        }
    }
    if incompatible_warned {
        println!();
    }

    let spinner2 = ui::Spinner::new("Installing...");
    let result = resolver.install(package)?;
    if result.success {
        spinner2.stop_with(&format!("{} {}", ui::icon_ok(), result.message));
    } else {
        spinner2.stop_with(&format!("{} {}", ui::icon_fail(), result.message));
    }

    println!();
    Ok(())
}

fn cmd_remove(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    println!();
    let spinner = ui::Spinner::new(&format!("Removing {package}..."));
    let result = resolver.remove(package)?;
    if result.success {
        spinner.stop_with(&format!("{} {}", ui::icon_ok(), result.message));
    } else {
        spinner.stop_with(&format!("{} {}", ui::icon_fail(), result.message));
    }
    println!();
    Ok(())
}

fn cmd_update(resolver: &Resolver) -> unvrs::error::Result<()> {
    println!();
    let spinner = ui::Spinner::new("Updating package lists...");
    let results = resolver.update()?;
    spinner.stop_with("Package lists updated");
    println!();
    for r in &results {
        if r.success {
            println!("  {} {}", ui::icon_ok(), r.message);
        } else {
            println!("  {} {}", ui::icon_fail(), r.message);
        }
    }
    println!();
    Ok(())
}

fn cmd_upgrade(resolver: &Resolver) -> unvrs::error::Result<()> {
    println!();
    let spinner = ui::Spinner::new("Upgrading packages...");
    let results = resolver.upgrade()?;
    spinner.stop_with("Upgrade complete");
    println!();
    for r in &results {
        if r.success {
            println!("  {} {}", ui::icon_ok(), r.message);
        } else {
            println!("  {} {}", ui::icon_fail(), r.message);
        }
    }
    println!();
    Ok(())
}

fn cmd_list(resolver: &Resolver) -> unvrs::error::Result<()> {
    let spinner = ui::Spinner::new("Listing installed packages...");
    let packages = resolver.list_installed()?;
    spinner.stop_with(&format!("Found {} packages", packages.len()));

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

fn cmd_doctor(resolver: &Resolver) -> unvrs::error::Result<()> {
    let os = resolver.os();

    println!();
    println!("  {}", ui::bold("unvrs doctor"));
    println!();

    // OS
    println!("  {}", ui::bold("System"));
    println!("  {} os       {} ({})", ui::icon_ok(), os.name, os.family);

    // Backends
    println!();
    println!("  {}", ui::bold("Backends"));
    for backend in resolver.registry().backends() {
        let compat = backend.is_compatible(os);
        let avail = backend.is_available();
        let has_caps = backend.capabilities().can_search;

        if avail && compat {
            println!(
                "  {} {:10} {}",
                ui::icon_ok(),
                backend.name(),
                ui::dim("ready")
            );
        } else if avail && !compat {
            println!(
                "  {} {:10} {}",
                ui::icon_warn(),
                backend.name(),
                ui::dim("wrong OS")
            );
        } else if has_caps {
            println!(
                "  {} {:10} {}",
                ui::icon_warn(),
                backend.name(),
                ui::dim("not installed")
            );
        } else {
            println!(
                "  {} {:10} {}",
                ui::icon_fail(),
                backend.name(),
                ui::dim("stub")
            );
        }
    }

    // Privileges
    println!();
    println!("  {}", ui::bold("Privileges"));
    if executor::is_root() {
        println!("  {} {}", ui::icon_ok(), ui::dim("root"));
    } else if executor::is_sudo_available() {
        println!("  {} {}", ui::icon_ok(), ui::dim("sudo available"));
    } else {
        println!("  {} {}", ui::icon_warn(), ui::dim("no sudo found"));
    }

    // Config
    println!();
    println!("  {}", ui::bold("Config"));
    let config_path = unvrs::config::Config::config_path();
    if config_path.exists() {
        println!("  {} {}", ui::icon_ok(), config_path.display());
    } else {
        println!(
            "  {} {}",
            ui::icon_warn(),
            ui::dim("no config file (using defaults)")
        );
    }

    println!();
    Ok(())
}
