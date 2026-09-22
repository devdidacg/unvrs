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
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for a package
    Search {
        /// Package name to search for
        package: String,
    },
    /// Show detailed information about a package
    Info {
        /// Package name
        package: String,
    },
    /// Install a package
    Install {
        /// Package name to install
        package: String,
    },
    /// Remove a package
    Remove {
        /// Package name to remove
        package: String,
    },
    /// Update package lists
    Update,
    /// Upgrade installed packages
    Upgrade,
    /// List installed packages
    List,
    /// Diagnose system configuration
    Doctor,
}
