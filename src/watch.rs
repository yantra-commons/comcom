use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::config::Config;
use crate::detection::{enumerate_ports, DeviceInfo};
use crate::display::print_watch_event;
use crate::history::{load_history, save_history, EventType, HistoryEvent};

// ─── Internal state ───────────────────────────────────────────────────────────

struct WatchEntry {
    board_name: String,
    connected_at: DateTime<Utc>,
    session_id: String,
}

// ─── Watch loop ───────────────────────────────────────────────────────────────

pub async fn watch_devices(filter: &str, interval_ms: u64, config: &Config) -> Result<()> {
    println!();
    println!(
        "{}",
        "  com-com  ·  Watching for Arduino devices".bold().on_bright_blue().white()
    );
    println!(
        "  {}  polling every {}ms — press {} to stop",
        "ℹ".cyan(),
        interval_ms,
        "Ctrl+C".bold()
    );
    println!("{}", "─".repeat(70).bright_black());
    println!();

    let mut history = load_history()?;

    // Port → tracking entry for every *currently connected* device
    let mut connected: HashMap<String, WatchEntry> = HashMap::new();
    // All ports that have been seen at any point this session (for "reconnected" detection)
    let mut ever_seen: HashSet<String> = HashSet::new();

    // Bootstrap: show devices already connected when we start
    let initial = enumerate_ports(filter).unwrap_or_default();
    for d in initial {
        let was_before = history.was_previously_seen(&d.port);
        let kind = if was_before { "reconnected" } else { "connected" };
        let nickname = config.nicknames.get(&d.port).map(|s| s.as_str());
        print_watch_event(kind, &d.port, &d.board_name, nickname, None);

        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        record_connection(&mut history, &d, &session_id, now)?;
        ever_seen.insert(d.port.clone());
        connected.insert(
            d.port.clone(),
            WatchEntry {
                board_name: d.board_name.clone(),
                connected_at: now,
                session_id,
            },
        );
    }
    if !connected.is_empty() {
        println!();
    }

    // ── Main polling loop ────────────────────────────────────────────────────
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;

        let current = match enumerate_ports(filter) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let current_map: HashMap<String, DeviceInfo> =
            current.into_iter().map(|d| (d.port.clone(), d)).collect();
        let current_ports: HashSet<String> = current_map.keys().cloned().collect();
        let known_ports: HashSet<String> = connected.keys().cloned().collect();

        // ── New connections ──────────────────────────────────────────────────
        for port in current_ports.difference(&known_ports) {
            let d = &current_map[port];
            let kind = if ever_seen.contains(port) {
                "reconnected"
            } else {
                "connected"
            };
            let nickname = config.nicknames.get(port).map(|s| s.as_str());
            print_watch_event(kind, port, &d.board_name, nickname, None);

            let session_id = Uuid::new_v4().to_string();
            let now = Utc::now();

            record_connection(&mut history, d, &session_id, now)?;
            ever_seen.insert(port.clone());
            connected.insert(
                port.clone(),
                WatchEntry {
                    board_name: d.board_name.clone(),
                    connected_at: now,
                    session_id,
                },
            );
        }

        // ── Disconnections ───────────────────────────────────────────────────
        let gone: Vec<String> = known_ports
            .difference(&current_ports)
            .cloned()
            .collect();

        for port in gone {
            let entry = connected.remove(&port).unwrap();
            let uptime = Utc::now()
                .signed_duration_since(entry.connected_at)
                .num_seconds();
            let nickname = config.nicknames.get(&port).map(|s| s.as_str());
            print_watch_event(
                "disconnected",
                &port,
                &entry.board_name,
                nickname,
                Some(uptime),
            );

            history.add_event(HistoryEvent {
                port: port.clone(),
                board_name: entry.board_name.clone(),
                vid: None,
                pid: None,
                event_type: EventType::Disconnected,
                timestamp: Utc::now(),
                session_id: entry.session_id.clone(),
                uptime_secs: Some(uptime),
            });
            save_history(&history)?;
        }
    }
}

// ─── Helper ───────────────────────────────────────────────────────────────────

fn record_connection(
    history: &mut crate::history::History,
    device: &DeviceInfo,
    session_id: &str,
    now: DateTime<Utc>,
) -> Result<()> {
    history.add_event(HistoryEvent {
        port: device.port.clone(),
        board_name: device.board_name.clone(),
        vid: device.vid,
        pid: device.pid,
        event_type: EventType::Connected,
        timestamp: now,
        session_id: session_id.to_string(),
        uptime_secs: None,
    });
    save_history(history)
}
