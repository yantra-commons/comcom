use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ─── Persistent config (nicknames, settings) ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// port → friendly name, e.g. "COM3" → "main-board"
    pub nicknames: HashMap<String, String>,
    /// Maximum days of history to retain (default: 30)
    pub history_max_days: Option<u32>,
    /// Maximum total history entries to keep
    pub history_max_entries: Option<usize>,
}

// ─── Session state (persists connection timestamps across short gaps) ─────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    /// port → session entry for every device currently known to be connected
    pub connected: HashMap<String, SessionDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDevice {
    pub connected_at: DateTime<Utc>,
    pub board_name: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub session_id: String,
}

// ─── Path helpers ─────────────────────────────────────────────────────────────

pub fn get_config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("Cannot determine the user config directory")?
        .join("com-com");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Cannot create config dir: {}", dir.display()))?;
    Ok(dir)
}

// ─── Config I/O ──────────────────────────────────────────────────────────────

pub fn load_config() -> Result<Config> {
    let path = get_config_dir()?.join("config.json");
    if path.exists() {
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_dir()?.join("config.json");
    std::fs::write(&path, serde_json::to_string_pretty(config)?)
        .with_context(|| format!("Cannot write {}", path.display()))
}

// ─── Session I/O ─────────────────────────────────────────────────────────────

pub fn load_session() -> Result<SessionState> {
    let path = get_config_dir()?.join("session.json");
    if path.exists() {
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    } else {
        Ok(SessionState::default())
    }
}

pub fn save_session(state: &SessionState) -> Result<()> {
    let path = get_config_dir()?.join("session.json");
    std::fs::write(&path, serde_json::to_string_pretty(state)?)
        .with_context(|| format!("Cannot write {}", path.display()))
}
