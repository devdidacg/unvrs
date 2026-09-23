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
    /// CPU architecture, e.g. `x86_64`, `aarch64`.
    pub arch: String,
    /// `ID_LIKE` values from os-release (e.g. mint -> ["ubuntu", "debian"]).
    pub id_like: Vec<String>,
}

impl OperatingSystem {
    /// Human label like `Ubuntu 24.04`.
    pub fn label(&self) -> String {
        match &self.version {
            Some(v) => format!("{} {}", self.name, v),
            None => self.name.clone(),
        }
    }
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
    /// Whether the backend can roll back a completed transaction.
    /// No current backend supports this; plans report it honestly.
    pub can_rollback: bool,
    /// Whether UNVRS can produce a dry-run plan for this backend
    /// (requires the backend to expose its command specs).
    pub dry_run: bool,
}

impl BackendCapabilities {
    /// Capabilities object with rollback always false and dry-run always true
    /// for backends that implement command specs.
    pub fn standard() -> Self {
        Self {
            can_search: true,
            can_info: true,
            can_install: true,
            can_remove: true,
            can_update: true,
            can_upgrade: true,
            can_list: true,
            can_rollback: false,
            dry_run: true,
        }
    }
}

/// Validate a package identifier before it is ever passed to a backend.
///
/// Allowed: ASCII letters, digits, and `@ . _ + - / : =`.
/// This blocks option-injection (`-rf`), shell metacharacters and whitespace.
/// Returns `Err(name)` when invalid.
pub fn validate_package_id(name: &str) -> std::result::Result<(), String> {
    if name.is_empty() {
        return Err(name.to_string());
    }
    if name.len() > 256 {
        return Err(name.to_string());
    }
    if name.starts_with('-') {
        return Err(name.to_string());
    }
    let ok = name.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '+' | '-' | '/' | ':' | '=')
    });
    if ok {
        Ok(())
    } else {
        Err(name.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct InstallationResult {
    pub success: bool,
    pub backend: String,
    pub package: String,
    pub message: String,
    /// Exit code of the underlying command, when one ran.
    pub exit_code: Option<i32>,
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

impl serde::Serialize for BackendCapabilities {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("BackendCapabilities", 9)?;
        state.serialize_field("search", &self.can_search)?;
        state.serialize_field("info", &self.can_info)?;
        state.serialize_field("install", &self.can_install)?;
        state.serialize_field("remove", &self.can_remove)?;
        state.serialize_field("update", &self.can_update)?;
        state.serialize_field("upgrade", &self.can_upgrade)?;
        state.serialize_field("list", &self.can_list)?;
        state.serialize_field("rollback", &self.can_rollback)?;
        state.serialize_field("dry_run", &self.dry_run)?;
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
