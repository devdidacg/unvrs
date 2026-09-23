use crate::package::{OperatingSystem, OsFamily};

pub fn detect() -> OperatingSystem {
    #[cfg(target_os = "windows")]
    {
        OperatingSystem {
            id: "windows".into(),
            name: "Windows".into(),
            version: windows_version(),
            family: OsFamily::Windows,
            arch: std::env::consts::ARCH.into(),
            id_like: Vec::new(),
        }
    }
    #[cfg(target_os = "macos")]
    {
        OperatingSystem {
            id: "macos".into(),
            name: "macOS".into(),
            version: macos_version(),
            family: OsFamily::MacOS,
            arch: std::env::consts::ARCH.into(),
            id_like: Vec::new(),
        }
    }
    #[cfg(target_os = "linux")]
    {
        linux_detect()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        OperatingSystem {
            id: "unknown".into(),
            name: "Unknown".into(),
            version: None,
            family: OsFamily::Unknown,
            arch: std::env::consts::ARCH.into(),
            id_like: Vec::new(),
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_version() -> Option<String> {
    std::env::var("OS")
        .ok()
        .filter(|v| v == "Windows_NT")
        .map(|_| "Windows NT".into())
}

#[cfg(target_os = "macos")]
fn macos_version() -> Option<String> {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })
}

#[cfg(target_os = "linux")]
fn linux_detect() -> OperatingSystem {
    let arch = std::env::consts::ARCH.to_string();
    if let Some(os_release) = parse_os_release() {
        OperatingSystem {
            id: os_release.id,
            name: os_release.name,
            version: os_release.version,
            family: OsFamily::Linux,
            arch,
            id_like: os_release.id_like,
        }
    } else {
        OperatingSystem {
            id: "linux".into(),
            name: "Linux".into(),
            version: None,
            family: OsFamily::Linux,
            arch,
            id_like: Vec::new(),
        }
    }
}

#[cfg(target_os = "linux")]
struct OsReleaseInfo {
    id: String,
    name: String,
    version: Option<String>,
    id_like: Vec<String>,
}

#[cfg(target_os = "linux")]
fn parse_os_release() -> Option<OsReleaseInfo> {
    let content = std::fs::read_to_string("/etc/os-release")
        .or_else(|_| std::fs::read_to_string("/usr/lib/os-release"))
        .ok()?;

    let mut id = String::new();
    let mut name = String::new();
    let mut version = None;
    let mut id_like = Vec::new();

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("ID=") {
            id = val.trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("NAME=") {
            name = val.trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
            version = Some(val.trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
            id_like = val
                .trim_matches('"')
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
        }
    }

    if id.is_empty() && name.is_empty() {
        return None;
    }

    Some(OsReleaseInfo {
        id,
        name,
        version,
        id_like,
    })
}

pub fn is_linux() -> bool {
    detect().family == OsFamily::Linux
}

pub fn is_unix() -> bool {
    cfg!(unix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_os() -> OperatingSystem {
        OperatingSystem {
            id: "ubuntu".into(),
            name: "Ubuntu".into(),
            version: Some("24.04".into()),
            family: OsFamily::Linux,
            arch: "x86_64".into(),
            id_like: Vec::new(),
        }
    }

    #[test]
    fn detect_runs() {
        let os = detect();
        assert!(!os.id.is_empty());
        assert!(!os.name.is_empty());
        assert!(!os.arch.is_empty());
    }

    #[test]
    fn os_family_display() {
        assert_eq!(OsFamily::Linux.to_string(), "Linux");
        assert_eq!(OsFamily::Windows.to_string(), "Windows");
        assert_eq!(OsFamily::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn os_label() {
        assert_eq!(sample_os().label(), "Ubuntu 24.04");
        let mut os = sample_os();
        os.version = None;
        assert_eq!(os.label(), "Ubuntu");
    }
}
