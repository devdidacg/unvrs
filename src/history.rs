use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// One recorded package-management transaction.
///
/// The shape is deliberately forward-compatible with a future rollback
/// feature: every field needed to *reproduce* or *reason about* an operation
/// is captured, and `rollback_supported` records (always honestly) whether
/// undo was possible.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    /// Monotonic transaction id (starts at 1). 0 = legacy entry.
    #[serde(default)]
    pub id: u64,
    pub timestamp: u64,
    pub action: String,
    pub package: String,
    pub backend: String,
    pub success: bool,
    /// What the user asked for (may differ from `package` after resolution).
    #[serde(default)]
    pub requested: String,
    /// The concrete package name handed to the backend.
    #[serde(default)]
    pub resolved: String,
    /// Exact command executed (display form).
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub rollback_supported: bool,
}

impl HistoryEntry {
    pub fn now(action: &str, requested: &str) -> Self {
        Self {
            id: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            action: action.to_string(),
            package: requested.to_string(),
            backend: String::new(),
            success: false,
            requested: requested.to_string(),
            resolved: String::new(),
            command: String::new(),
            exit_code: None,
            warnings: Vec::new(),
            rollback_supported: false,
        }
    }
}

const MAX_ENTRIES: usize = 500;

pub fn history_path() -> PathBuf {
    crate::config::data_dir().join("history.json")
}

pub fn load() -> Vec<HistoryEntry> {
    let path = history_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn get(id: u64) -> Option<HistoryEntry> {
    load().into_iter().find(|e| e.id == id)
}

/// Append a transaction, assigning the next id. Dry-runs are never recorded
/// (callers must not record them) — history stays a log of real changes.
pub fn record(mut entry: HistoryEntry) -> u64 {
    let path = history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let mut entries = load();
    let next = entries.iter().map(|e| e.id).max().unwrap_or(0) + 1;
    entry.id = next;
    entries.push(entry);

    if entries.len() > MAX_ENTRIES {
        entries = entries.split_off(entries.len() - MAX_ENTRIES);
    }

    if let Ok(json) = serde_json::to_string_pretty(&entries) {
        fs::write(&path, json).ok();
    }
    next
}

/// Backward-compatible helper (legacy call sites).
pub fn add_entry(action: &str, package: &str, backend: &str, success: bool) -> u64 {
    let mut e = HistoryEntry::now(action, package);
    e.backend = backend.to_string();
    e.success = success;
    e.package = package.to_string();
    record(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_entries_deserialize_with_defaults() {
        let legacy = r#"[{
            "timestamp": 1700000000,
            "action": "install",
            "package": "git",
            "backend": "apt",
            "success": true
        }]"#;
        let entries: Vec<HistoryEntry> = serde_json::from_str(legacy).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 0);
        assert_eq!(entries[0].package, "git");
        assert!(entries[0].command.is_empty());
        assert!(entries[0].warnings.is_empty());
        assert!(!entries[0].rollback_supported);
    }

    #[test]
    fn entry_serializes_all_transaction_fields() {
        let mut e = HistoryEntry::now("install", "neovim");
        e.id = 42;
        e.backend = "apt".into();
        e.success = true;
        e.resolved = "neovim".into();
        e.command = "apt-get install -y neovim".into();
        e.exit_code = Some(0);
        e.warnings = vec!["requires root".into()];
        let json = serde_json::to_string(&e).unwrap();
        for key in [
            "\"id\":42",
            "\"requested\"",
            "\"resolved\"",
            "\"command\"",
            "\"exit_code\"",
            "\"warnings\"",
            "\"rollback_supported\"",
        ] {
            assert!(json.contains(key), "missing {key} in {json}");
        }
    }

    #[test]
    fn history_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        let entries = vec![{
            let mut e = HistoryEntry::now("install", "curl");
            e.id = 1;
            e.backend = "apt".into();
            e.success = true;
            e
        }];
        fs::write(&path, serde_json::to_string_pretty(&entries).unwrap()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let parsed: Vec<HistoryEntry> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, 1);
        assert_eq!(parsed[0].requested, "curl");
    }
}
