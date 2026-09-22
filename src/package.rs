use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageSource {
    System,
    Flatpak,
    Snap,
    Manual,
}

impl fmt::Display for PackageSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageSource::System => write!(f, "system"),
            PackageSource::Flatpak => write!(f, "flatpak"),
            PackageSource::Snap => write!(f, "snap"),
            PackageSource::Manual => write!(f, "manual"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageCandidate {
    pub name: String,
    pub version: Option<String>,
    pub source: PackageSource,
    pub backend: String,
    pub architecture: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: Option<String>,
    pub source: PackageSource,
    pub backend: String,
    pub architecture: Option<String>,
    pub description: Option<String>,
    pub maintainer: Option<String>,
    pub homepage: Option<String>,
    pub dependencies: Vec<String>,
    pub installed_size: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub source: PackageSource,
    pub backend: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatingSystem {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub family: OsFamily,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OsFamily {
    Linux,
    BSD,
    MacOS,
    Windows,
    Unknown,
}

impl fmt::Display for OsFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OsFamily::Linux => write!(f, "Linux"),
            OsFamily::BSD => write!(f, "BSD"),
            OsFamily::MacOS => write!(f, "macOS"),
            OsFamily::Windows => write!(f, "Windows"),
            OsFamily::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub can_search: bool,
    pub can_info: bool,
    pub can_install: bool,
    pub can_remove: bool,
    pub can_update: bool,
    pub can_upgrade: bool,
    pub can_list: bool,
}

#[derive(Debug, Clone)]
pub struct InstallationResult {
    pub success: bool,
    pub backend: String,
    pub package: String,
    pub message: String,
}
