use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "unvrs",
    about = "Universal package manager CLI — one interface over many package managers",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for a package
    #[command(short_flag = 's')]
    Search {
        /// Package name to search for
        package: String,
        /// Search only in a specific backend
        #[arg(short, long)]
        backend: Option<String>,
    },
    /// Show detailed information about a package
    #[command(short_flag = 'I')]
    Info {
        /// Package name
        package: String,
    },
    /// Install a package
    #[command(short_flag = 'i')]
    Install {
        /// Package name to install
        package: String,
        /// Simulate installation without making changes
        #[arg(long)]
        dry: bool,
        /// Force installation from any backend (including cross-distro)
        #[arg(long)]
        force: bool,
    },
    /// Remove a package
    #[command(short_flag = 'r')]
    Remove {
        /// Package name to remove
        package: String,
    },
    /// Update package lists
    #[command(short_flag = 'U')]
    Update,
    /// Upgrade installed packages
    #[command(short_flag = 'u')]
    Upgrade,
    /// List installed packages
    #[command(short_flag = 'l')]
    List,
    /// Show packages with available updates
    Outdated,
    /// Show installation history
    History,
    /// Clean system package cache
    Clean,
    /// Diagnose system configuration
    Doctor,
}
