use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "unvrs",
    about = "\n  Universal package manager CLI\n  One interface over many package managers\n",
    version,
    propagate_version = true,
    after_help = "EXAMPLES:\n  unvrs search firefox                 Search across all backends\n  unvrs search --backend pacman neovim Search only in pacman\n  sudo unvrs install fish vim          Install packages from best backends\n  unvrs install --dry vim              Simulate an installation (plan only)\n  unvrs install --backend apt git      Install git via apt explicitly\n  unvrs plan install firefox           Show the full plan, change nothing\n  unvrs apply dev                      Apply a package profile\n  unvrs list                           List installed packages\n  unvrs doctor                         Diagnose your system\n\nDOCUMENTATION:\n  https://github.com/devdidacg/unvrs\n"
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

    /// Increase logging verbosity (-v, -vv, -vvv); logs go to stderr
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for a package across all backends
    #[command(
        short_flag = 's',
        after_help = "EXAMPLES:\n  unvrs search firefox\n  unvrs search --backend apt vim\n"
    )]
    Search {
        /// Package name to search for
        package: String,

        /// Search only in a specific backend
        #[arg(short, long)]
        backend: Option<String>,
    },

    /// Show detailed information about a package
    #[command(short_flag = 'I', after_help = "EXAMPLE:\n  unvrs info git\n")]
    Info {
        /// Package name
        package: String,
    },

    /// Install one or more packages
    #[command(
        short_flag = 'i',
        after_help = "EXAMPLES:\n  sudo unvrs install fish vim\n  unvrs install --dry vim\n  unvrs install --backend apt git\n  unvrs install --container vim\n  unvrs install --explain fish\n"
    )]
    Install {
        /// Package name(s) to install
        #[arg(required = true, num_args = 1..)]
        packages: Vec<String>,

        /// Simulate: resolve and print the plan without changing anything
        #[arg(long)]
        dry: bool,

        /// Install via a specific backend (skips auto-resolution)
        #[arg(short, long)]
        backend: Option<String>,

        /// Allow a non-native (cross-distro) backend to run directly
        #[arg(long)]
        cross_distro: bool,

        /// Deprecated alias for --cross-distro
        #[arg(long, hide = true)]
        force: bool,

        /// Prefer an available container backend (Docker/Podman)
        #[arg(long)]
        container: bool,

        /// Print the full resolution report before acting
        #[arg(long)]
        explain: bool,
    },

    /// Remove one or more installed packages
    #[command(
        short_flag = 'r',
        after_help = "EXAMPLES:\n  sudo unvrs remove fish\n  unvrs remove --dry vim\n  unvrs remove --explain fish\n"
    )]
    Remove {
        /// Package name(s) to remove
        #[arg(required = true, num_args = 1..)]
        packages: Vec<String>,

        /// Simulate: resolve and print the plan without changing anything
        #[arg(long)]
        dry: bool,

        /// Remove via a specific backend
        #[arg(short, long)]
        backend: Option<String>,

        /// Allow a non-native (cross-distro) backend
        #[arg(long)]
        cross_distro: bool,

        /// Print the full resolution report before acting
        #[arg(long)]
        explain: bool,
    },

    /// Resolve packages and print the exact plan (changes nothing)
    #[command(
        after_help = "EXAMPLES:\n  unvrs plan install firefox vim\n  unvrs plan remove firefox\n  unvrs --json plan install vim\n"
    )]
    Plan {
        #[command(subcommand)]
        operation: PlanOperation,
    },

    /// Apply a named package profile
    #[command(after_help = "EXAMPLES:\n  unvrs apply dev --dry\n  sudo unvrs apply dev\n")]
    Apply {
        /// Profile name
        profile: String,

        /// Simulate: print the plan without changing anything
        #[arg(long)]
        dry: bool,

        /// Print the full resolution report before acting
        #[arg(long)]
        explain: bool,

        /// Override the profile's backend
        #[arg(short, long)]
        backend: Option<String>,
    },

    /// Manage named package profiles
    #[command(
        after_help = "EXAMPLE:\n  unvrs profile create dev git neovim ripgrep\n  unvrs profile list\n  unvrs profile plan dev\n  unvrs apply dev\n"
    )]
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
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
        after_help = "EXAMPLES:\n  unvrs list\n  unvrs --json list\n"
    )]
    List,

    /// Show packages with available updates
    #[command(after_help = "EXAMPLE:\n  unvrs outdated\n")]
    Outdated,

    /// Show transaction history
    #[command(
        after_help = "EXAMPLES:\n  unvrs history\n  unvrs history show 42\n  unvrs --json history\n"
    )]
    History {
        #[command(subcommand)]
        action: Option<HistoryAction>,
    },

    /// Clean system package cache
    #[command(after_help = "EXAMPLE:\n  sudo unvrs clean\n")]
    Clean,

    /// Diagnose system configuration
    #[command(after_help = "EXAMPLES:\n  unvrs doctor\n  unvrs --json doctor\n")]
    Doctor,
}

#[derive(Subcommand, Debug)]
pub enum PlanOperation {
    /// Plan an installation
    Install {
        #[arg(required = true, num_args = 1..)]
        packages: Vec<String>,

        #[arg(short, long)]
        backend: Option<String>,

        #[arg(long)]
        cross_distro: bool,

        #[arg(long)]
        container: bool,
    },

    /// Plan a removal
    Remove {
        #[arg(required = true, num_args = 1..)]
        packages: Vec<String>,

        #[arg(short, long)]
        backend: Option<String>,

        #[arg(long)]
        cross_distro: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileAction {
    /// Create a new profile
    Create {
        /// Profile name (letters, digits, - and _)
        name: String,

        /// Packages to include
        #[arg(required = true, num_args = 1..)]
        packages: Vec<String>,
    },

    /// List all profiles
    List,

    /// Show a profile's contents
    Show { name: String },

    /// Print the plan for applying a profile (changes nothing)
    Plan { name: String },

    /// Apply a profile (same as `unvrs apply <name>`)
    Apply { name: String },
}

#[derive(Subcommand, Debug)]
pub enum HistoryAction {
    /// Show one transaction in detail
    Show { id: u64 },
}
