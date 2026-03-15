use chrono::Utc;
use colored::Colorize;
use std::collections::HashMap;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::config::{Config, SessionDevice};
use crate::detection::DeviceInfo;
use crate::history::{format_local, format_relative, format_uptime, History};

// ─── Table row structs ────────────────────────────────────────────────────────

#[derive(Tabled)]
struct DeviceRow {
    #[tabled(rename = "Port")]
    port: String,
    #[tabled(rename = "Board")]
    board: String,
    #[tabled(rename = "Nickname")]
    nickname: String,
    #[tabled(rename = "VID:PID")]
    vid_pid: String,
    #[tabled(rename = "Manufacturer")]
    manufacturer: String,
    #[tabled(rename = "Connected At")]
    connected_at: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
}

#[derive(Tabled)]
struct HistoryRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Event")]
    event: String,
    #[tabled(rename = "Port")]
    port: String,
    #[tabled(rename = "Board")]
    board: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
}

// ─── Header ──────────────────────────────────────────────────────────────────

pub fn print_header(title: &str) {
    println!();
    println!("{}", format!("  {}  ", title).bold().on_bright_blue().white());
    println!("{}", "─".repeat(70).bright_black());
}

// ─── Device list ─────────────────────────────────────────────────────────────

pub fn print_device_list(
    devices: &[DeviceInfo],
    config: &Config,
    session: &HashMap<String, SessionDevice>,
    relative_time: bool,
    verbose: bool,
) {
    print_header("com-com  ·  Arduino COM Port Manager");

    if devices.is_empty() {
        println!(
            "  {}  No devices found.",
            "⚠".yellow()
        );
        println!(
            "  {}",
            "Connect an Arduino or microcontroller and try again.".bright_black()
        );
        println!();
        return;
    }

    let now = Utc::now();

    let rows: Vec<DeviceRow> = devices
        .iter()
        .map(|d| {
            let nickname = config.nicknames.get(&d.port).cloned().unwrap_or_default();

            let (connected_at, uptime) = match session.get(&d.port) {
                Some(sess) => {
                    let secs = now.signed_duration_since(sess.connected_at).num_seconds();
                    let time_str = if relative_time {
                        format_relative(sess.connected_at)
                    } else {
                        format_local(sess.connected_at)
                    };
                    (time_str, format_uptime(secs))
                }
                None => ("—".to_string(), "—".to_string()),
            };

            DeviceRow {
                port: d.port.clone(),
                board: d.board_name.clone(),
                nickname,
                vid_pid: d.vid_pid_str(),
                manufacturer: d.manufacturer.clone(),
                connected_at,
                uptime,
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::modern());
    println!("{}", table);

    // Summary line
    let arduino_n = devices.iter().filter(|d| d.is_arduino).count();
    let total = devices.len();
    println!();
    println!(
        "  {} device(s) total  ·  {} Arduino/MCU  ·  {} other",
        total.to_string().bold(),
        arduino_n.to_string().green().bold(),
        (total - arduino_n).to_string().yellow(),
    );

    if verbose {
        println!();
        println!("{}", "  Verbose details:".bold());
        println!("{}", "  ─".repeat(35).bright_black());
        for d in devices {
            let tag = if d.is_arduino {
                "[MCU]".green().to_string()
            } else {
                "[serial]".yellow().to_string()
            };
            println!("  {} {} {}", d.port.bold().cyan(), d.board_name.bold(), tag);
            if let Some(sn) = &d.serial_number {
                println!("      Serial #  : {}", sn);
            }
            if let Some(prod) = &d.product {
                println!("      Product   : {}", prod);
            }
        }
    }

    println!();
}

// ─── History ─────────────────────────────────────────────────────────────────

pub fn print_history(history: &History, days: u32, relative_time: bool) {
    print_header(&format!("com-com  ·  History (last {} day(s))", days));

    let events = history.events_since(days);
    if events.is_empty() {
        println!("  {} No history in the last {} day(s).", "ℹ".cyan(), days);
        println!();
        return;
    }

    let rows: Vec<HistoryRow> = events
        .iter()
        .map(|e| {
            let time_str = if relative_time {
                format_relative(e.timestamp)
            } else {
                format_local(e.timestamp)
            };
            let event_str = match e.event_type {
                crate::history::EventType::Connected => "Connected",
                crate::history::EventType::Disconnected => "Disconnected",
            }
            .to_string();
            let uptime = e
                .uptime_secs
                .map(|s| format_uptime(s))
                .unwrap_or_else(|| "—".to_string());

            HistoryRow {
                time: time_str,
                event: event_str,
                port: e.port.clone(),
                board: e.board_name.clone(),
                uptime,
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::modern());
    println!("{}", table);
    println!();
    println!("  {} event(s) shown.", events.len().to_string().bold());
    println!();
}

// ─── Recently disconnected ────────────────────────────────────────────────────

pub fn print_recent_disconnected(history: &History, timeout_mins: i64) {
    print_header("com-com  ·  Recently Disconnected");

    let events = history.recently_disconnected(timeout_mins);
    if events.is_empty() {
        println!(
            "  {} No devices disconnected in the last {} minute(s).",
            "ℹ".cyan(),
            timeout_mins
        );
        println!();
        return;
    }

    for e in events {
        let ago = format_relative(e.timestamp);
        let uptime_str = e
            .uptime_secs
            .map(|s| format!("  (was connected for {})", format_uptime(s)))
            .unwrap_or_default();
        println!(
            "  {}  {}  {}  — disconnected {}{}",
            "✗".red().bold(),
            e.port.bold().red(),
            e.board_name.bright_black(),
            ago.yellow(),
            uptime_str.bright_black(),
        );
    }
    println!();
}

// ─── Nicknames list ───────────────────────────────────────────────────────────

pub fn print_nicknames(config: &Config) {
    print_header("com-com  ·  Saved Nicknames");

    if config.nicknames.is_empty() {
        println!("  {} No nicknames saved.", "ℹ".cyan());
        println!(
            "  {}",
            "Use: com-com --nickname <PORT> <NAME>".bright_black()
        );
        println!();
        return;
    }

    let mut pairs: Vec<(&String, &String)> = config.nicknames.iter().collect();
    pairs.sort_by_key(|(k, _)| k.as_str());
    for (port, name) in pairs {
        println!("  {}  →  {}", port.bold().cyan(), name.bold().green());
    }
    println!();
}

// ─── Watch event lines ────────────────────────────────────────────────────────

pub fn print_watch_event(
    kind: &str, // "connected" | "disconnected" | "reconnected"
    port: &str,
    board: &str,
    nickname: Option<&str>,
    uptime_secs: Option<i64>,
) {
    use chrono::Local;
    let ts = Local::now().format("%H:%M:%S").to_string();
    let nick = nickname
        .map(|n| format!(" [{}]", n))
        .unwrap_or_default();
    let uptime = uptime_secs
        .map(|s| format!("  (uptime {})", format_uptime(s)))
        .unwrap_or_default();

    match kind {
        "connected" => println!(
            "[{}]  {}  {}  ·  {}{}",
            ts.bright_black(),
            "✔  Connected   ".green().bold(),
            port.bold().cyan(),
            board,
            nick.yellow(),
        ),
        "disconnected" => println!(
            "[{}]  {}  {}  ·  {}{}{}",
            ts.bright_black(),
            "✘  Disconnected".red().bold(),
            port.bold().cyan(),
            board,
            nick.yellow(),
            uptime.bright_black(),
        ),
        "reconnected" => println!(
            "[{}]  {}  {}  ·  {}{}",
            ts.bright_black(),
            "↺  Reconnected ".bright_green().bold(),
            port.bold().cyan(),
            board,
            nick.yellow(),
        ),
        _ => {}
    }
}
