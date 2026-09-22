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

#[derive(Debug, Clone)]
pub struct OutdatedPackage {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub backend: String,
}

impl serde::Serialize for OutdatedPackage {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("OutdatedPackage", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("current_version", &self.current_version)?;
        state.serialize_field("latest_version", &self.latest_version)?;
        state.serialize_field("backend", &self.backend)?;
        state.end()
    }
}

impl serde::Serialize for PackageCandidate {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PackageCandidate", 6)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("source", &self.source.to_string())?;
        state.serialize_field("backend", &self.backend)?;
        state.serialize_field("architecture", &self.architecture)?;
        state.serialize_field("description", &self.description)?;
        state.end()
    }
}

impl serde::Serialize for PackageInfo {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PackageInfo", 10)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("source", &self.source.to_string())?;
        state.serialize_field("backend", &self.backend)?;
        state.serialize_field("architecture", &self.architecture)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("maintainer", &self.maintainer)?;
        state.serialize_field("homepage", &self.homepage)?;
        state.serialize_field("dependencies", &self.dependencies)?;
        state.serialize_field("installed_size", &self.installed_size)?;
        state.end()
    }
}

impl serde::Serialize for InstalledPackage {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("InstalledPackage", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("source", &self.source.to_string())?;
        state.serialize_field("backend", &self.backend)?;
        state.end()
    }
}
