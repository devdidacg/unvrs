use clap::Parser;
use unvrs::cli::{Cli, Commands};
use unvrs::config::Config;
use unvrs::executor;
use unvrs::resolver::{Resolver, SearchStatus};

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
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn cmd_search(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    println!("\nunvrs\n");
    println!("Searching for {package}...\n");

    let results = resolver.search_with_status(package);

    for (backend_name, status) in &results {
        match status {
            SearchStatus::Found(_) => println!("  \x1b[32m✓\x1b[0m {backend_name}"),
            SearchStatus::NotFound => println!("  \x1b[31m✗\x1b[0m {backend_name}"),
            SearchStatus::NotImplemented => {
                println!("  \x1b[33m!\x1b[0m {backend_name} (not implemented)")
            }
            SearchStatus::Error => println!("  \x1b[31m✗\x1b[0m {backend_name} (error)"),
        }
    }

    let mut found_any = false;
    for (backend_name, status) in &results {
        if let SearchStatus::Found(candidates) = status {
            if !found_any {
                println!("\nFound:\n");
                found_any = true;
            }
            for c in candidates {
                print!("  {} ", c.name);
                if let Some(v) = &c.version {
                    print!("{v} ");
                }
                println!("(source: {backend_name})");
                if let Some(desc) = &c.description {
                    println!("    {desc}");
                }
            }
        }
    }

    if !found_any {
        println!("\nNo packages found for '{package}'.");
    }

    println!();
    Ok(())
}

fn cmd_info(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    println!("\nunvrs\n");
    println!("Looking up info for {package}...\n");

    match resolver.info(package)? {
        Some(info) => {
            println!("Name:          {}", info.name);
            if let Some(v) = &info.version {
                println!("Version:       {v}");
            }
            if let Some(d) = &info.description {
                println!("Description:   {d}");
            }
            if let Some(a) = &info.architecture {
                println!("Architecture:  {a}");
            }
            if let Some(m) = &info.maintainer {
                println!("Maintainer:    {m}");
            }
            if let Some(h) = &info.homepage {
                println!("Homepage:      {h}");
            }
            if let Some(s) = &info.installed_size {
                println!("Installed:     {s}");
            }
            if !info.dependencies.is_empty() {
                println!("Dependencies:  {}", info.dependencies.join(", "));
            }
            println!("Backend:       {}", info.backend);
        }
        None => {
            println!("No information found for '{package}'.");
        }
    }

    println!();
    Ok(())
}

fn cmd_install(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    println!("\nunvrs\n");

    if !executor::is_root() && executor::is_sudo_available() {
        println!("Note: this operation may require root privileges.\n");
    }

    println!("Searching for {package}...\n");

    let results = resolver.search_with_status(package);

    for (backend_name, status) in &results {
        match status {
            SearchStatus::Found(_) => println!("  \x1b[32m✓\x1b[0m {backend_name}"),
            SearchStatus::NotFound => println!("  \x1b[31m✗\x1b[0m {backend_name}"),
            SearchStatus::NotImplemented => {
                println!("  \x1b[33m!\x1b[0m {backend_name} (not implemented)")
            }
            SearchStatus::Error => println!("  \x1b[31m✗\x1b[0m {backend_name} (error)"),
        }
    }

    println!("\nInstalling...\n");

    let result = resolver.install(package)?;
    if result.success {
        println!("\x1b[32m✓\x1b[0m {}", result.message);
    } else {
        println!("\x1b[31m✗\x1b[0m {}", result.message);
    }

    println!();
    Ok(())
}

fn cmd_remove(resolver: &Resolver, package: &str) -> unvrs::error::Result<()> {
    println!("\nunvrs\n");
    println!("Removing {package}...\n");

    let result = resolver.remove(package)?;
    if result.success {
        println!("\x1b[32m✓\x1b[0m {}", result.message);
    } else {
        println!("\x1b[31m✗\x1b[0m {}", result.message);
    }

    println!();
    Ok(())
}

fn cmd_update(resolver: &Resolver) -> unvrs::error::Result<()> {
    println!("\nunvrs\n");
    println!("Updating package lists...\n");

    let results = resolver.update()?;
    for r in &results {
        if r.success {
            println!("\x1b[32m✓\x1b[0m {}", r.message);
        } else {
            println!("\x1b[31m✗\x1b[0m {}", r.message);
        }
    }

    println!();
    Ok(())
}

fn cmd_upgrade(resolver: &Resolver) -> unvrs::error::Result<()> {
    println!("\nunvrs\n");
    println!("Upgrading packages...\n");

    let results = resolver.upgrade()?;
    for r in &results {
        if r.success {
            println!("\x1b[32m✓\x1b[0m {}", r.message);
        } else {
            println!("\x1b[31m✗\x1b[0m {}", r.message);
        }
    }

    println!();
    Ok(())
}

fn cmd_list(resolver: &Resolver) -> unvrs::error::Result<()> {
    println!("\nunvrs\n");
    println!("Installed packages:\n");

    let packages = resolver.list_installed()?;
    if packages.is_empty() {
        println!("  (no packages found via available backends)");
    } else {
        for p in &packages {
            println!("  {} {} ({})", p.name, p.version, p.backend);
        }
    }

    println!();
    Ok(())
}

fn cmd_doctor(resolver: &Resolver) -> unvrs::error::Result<()> {
    let os = resolver.os();
    println!("\nunvrs doctor\n");

    println!("Operating system:");
    println!("  \x1b[32m✓\x1b[0m {} ({})", os.name, os.family);

    println!("\nPackage manager backends:");
    for backend in resolver.registry().backends() {
        let compat = backend.is_compatible(os);
        let avail = backend.is_available();
        if avail && compat {
            println!("  \x1b[32m✓\x1b[0m {} (available)", backend.name());
        } else if !backend.capabilities().can_search && avail {
            println!(
                "  \x1b[33m!\x1b[0m {} (detected but not implemented)",
                backend.name()
            );
        } else {
            println!("  \x1b[31m✗\x1b[0m {}", backend.name());
        }
    }

    println!("\nPrivileges:");
    if executor::is_root() {
        println!("  \x1b[32m✓\x1b[0m running as root");
    } else if executor::is_sudo_available() {
        println!("  \x1b[32m✓\x1b[0m normal user (sudo available)");
    } else {
        println!("  \x1b[33m!\x1b[0m normal user (sudo not found)");
    }

    println!("\nConfiguration:");
    let config_path = unvrs::config::Config::config_path();
    if config_path.exists() {
        println!("  \x1b[32m✓\x1b[0m {}", config_path.display());
    } else {
        println!("  \x1b[33m!\x1b[0m no config file (using defaults)");
    }

    println!();
    Ok(())
}
