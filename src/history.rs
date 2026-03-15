use anyhow::Result;
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};

use crate::config::get_config_dir;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    Connected,
    Disconnected,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Connected => write!(f, "Connected"),
            EventType::Disconnected => write!(f, "Disconnected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub port: String,
    pub board_name: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    /// Total uptime in seconds; only set for `Disconnected` events.
    pub uptime_secs: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    pub events: Vec<HistoryEvent>,
}

impl History {
    pub fn add_event(&mut self, event: HistoryEvent) {
        self.events.push(event);
    }

    pub fn events_since(&self, days: u32) -> Vec<&HistoryEvent> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        self.events.iter().filter(|e| e.timestamp >= cutoff).collect()
    }

    pub fn recently_disconnected(&self, timeout_mins: i64) -> Vec<&HistoryEvent> {
        let cutoff = Utc::now() - Duration::minutes(timeout_mins);
        self.events
            .iter()
            .filter(|e| e.event_type == EventType::Disconnected && e.timestamp >= cutoff)
            .collect()
    }

    pub fn was_previously_seen(&self, port: &str) -> bool {
        self.events.iter().any(|e| e.port == port)
    }

    #[allow(dead_code)]
    pub fn cleanup(&mut self, max_days: u32) {
        let cutoff = Utc::now() - Duration::days(max_days as i64);
        self.events.retain(|e| e.timestamp >= cutoff);
    }
}

// ─── Persistence ─────────────────────────────────────────────────────────────

pub fn load_history() -> Result<History> {
    let path = get_config_dir()?.join("history.json");
    if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    } else {
        Ok(History::default())
    }
}

pub fn save_history(history: &History) -> Result<()> {
    let path = get_config_dir()?.join("history.json");
    std::fs::write(path, serde_json::to_string_pretty(history)?)?;
    Ok(())
}

// ─── Formatting helpers ───────────────────────────────────────────────────────

pub fn format_uptime(secs: i64) -> String {
    let secs = secs.max(0);
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3_600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86_400 {
        let h = secs / 3_600;
        let m = (secs % 3_600) / 60;
        format!("{}h {}m", h, m)
    } else {
        let d = secs / 86_400;
        let h = (secs % 86_400) / 3_600;
        format!("{}d {}h", d, h)
    }
}

pub fn format_relative(dt: DateTime<Utc>) -> String {
    let secs = Utc::now().signed_duration_since(dt).num_seconds().max(0);
    if secs < 5 {
        "just now".to_string()
    } else if secs < 60 {
        format!("{} seconds ago", secs)
    } else if secs < 3_600 {
        let m = secs / 60;
        if m == 1 { "1 minute ago".to_string() } else { format!("{} minutes ago", m) }
    } else if secs < 86_400 {
        let h = secs / 3_600;
        if h == 1 { "1 hour ago".to_string() } else { format!("{} hours ago", h) }
    } else {
        let d = secs / 86_400;
        if d == 1 { "1 day ago".to_string() } else { format!("{} days ago", d) }
    }
}

/// Format a UTC timestamp as a local-time string.
pub fn format_local(dt: DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.into();
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}
