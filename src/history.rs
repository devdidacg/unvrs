use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub timestamp: u64,
    pub action: String,
    pub package: String,
    pub backend: String,
    pub success: bool,
}

pub fn history_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unvrs")
        .join("history.json")
}

pub fn add_entry(action: &str, package: &str, backend: &str, success: bool) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let mut entries = load();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    entries.push(HistoryEntry {
        timestamp,
        action: action.to_string(),
        package: package.to_string(),
        backend: backend.to_string(),
        success,
    });

    // Keep last 100 entries
    if entries.len() > 100 {
        entries = entries.split_off(entries.len() - 100);
    }

    if let Ok(json) = serde_json::to_string_pretty(&entries) {
        fs::write(&path, json).ok();
    }
}

pub fn load() -> Vec<HistoryEntry> {
    let path = history_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}
