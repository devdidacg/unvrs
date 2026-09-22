use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "unvrs",
    about = "\n  Universal package manager CLI\n  One interface over many package managers\n",
    version,
    propagate_version = true,
    after_help = "EXAMPLES:\n  unvrs search firefox              Search across all backends\n  unvrs search -s pacman neovim     Search only in pacman\n  sudo unvrs install fish           Install a package\n  sudo unvrs install --dry vim      Simulate installation\n  sudo unvrs install --force apt git Install from apt via Docker\n  unvrs list                        List installed packages\n  unvrs doctor                      Diagnose your system\n\nDOCUMENTATION:\n  https://github.com/devdidacg/unvrs\n"
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
    /// Search for a package across all backends
    #[command(
        short_flag = 's',
        after_help = "EXAMPLES:\n  unvrs search firefox              Search in all backends\n  unvrs search -s pacman neovim     Search only in pacman\n  unvrs search --backend apt vim    Search only in apt\n"
    )]
    Search {
        /// Package name to search for
        package: String,

        /// Search only in a specific backend
        #[arg(short, long)]
        backend: Option<String>,
    },

    /// Show detailed information about a package
    #[command(
        short_flag = 'I',
        after_help = "EXAMPLE:\n  unvrs info git\n  unvrs info -I neovim\n"
    )]
    Info {
        /// Package name
        package: String,
    },

    /// Install a package
    #[command(
        short_flag = 'i',
        after_help = "EXAMPLES:\n  sudo unvrs install fish           Install from best backend\n  sudo unvrs install --dry vim      Simulate installation\n  sudo unvrs install --force apt git Install from apt via Docker\n  sudo unvrs install --container vim Force container installation\n"
    )]
    Install {
        /// Package name to install
        package: String,

        /// Simulate installation without making changes
        #[arg(long)]
        dry: bool,

        /// Force installation from any backend (including cross-distro)
        #[arg(long)]
        force: bool,

        /// Force installation via container backend (Docker/Podman)
        #[arg(long)]
        container: bool,
    },

    /// Remove a package
    #[command(short_flag = 'r', after_help = "EXAMPLE:\n  sudo unvrs remove fish\n")]
    Remove {
        /// Package name to remove
        package: String,
    },

    /// Update package lists
    #[command(short_flag = 'U', after_help = "EXAMPLE:\n  sudo unvrs update\n")]
    Update,

    /// Upgrade installed packages
    #[command(short_flag = 'u', after_help = "EXAMPLE:\n  sudo unvrs upgrade\n")]
    Upgrade,

    /// List installed packages
    #[command(
        short_flag = 'l',
        after_help = "EXAMPLES:\n  unvrs list                       List all installed packages\n  unvrs --json list                Output as JSON\n"
    )]
    List,

    /// Show packages with available updates
    #[command(after_help = "EXAMPLE:\n  unvrs outdated\n")]
    Outdated,

    /// Show installation history
    #[command(after_help = "EXAMPLE:\n  unvrs history\n")]
    History,

    /// Clean system package cache
    #[command(after_help = "EXAMPLE:\n  sudo unvrs clean\n")]
    Clean,

    /// Diagnose system configuration
    #[command(
        after_help = "EXAMPLES:\n  unvrs doctor                     Show all backends and status\n  unvrs --json doctor              Output as JSON\n"
    )]
    Doctor,
}
