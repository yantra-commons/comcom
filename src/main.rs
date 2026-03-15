use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use colored::Colorize;
use std::io::{self, Write};
use uuid::Uuid;

mod cli;
mod config;
mod detection;
mod display;
mod export;
mod history;
mod watch;

use cli::Cli;
use config::SessionDevice;
use history::{EventType, HistoryEvent};

#[tokio::main]
async fn main() -> Result<()> {
    // Enable ANSI colour on older Windows consoles (no-op on others)
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap_or(());

    let cli = Cli::parse();

    // ── Nickname management ──────────────────────────────────────────────────

    if let Some(ref parts) = cli.nickname {
        let port = &parts[0];
        let name = &parts[1];
        let mut cfg = config::load_config()?;
        cfg.nicknames.insert(port.clone(), name.clone());
        config::save_config(&cfg)?;
        println!(
            "  {}  Nickname for {} set to {}",
            "✔".green().bold(),
            port.bold().cyan(),
            name.bold().green()
        );
        return Ok(());
    }

    if let Some(ref port) = cli.remove_nickname {
        let mut cfg = config::load_config()?;
        if cfg.nicknames.remove(port).is_some() {
            config::save_config(&cfg)?;
            println!("  {}  Removed nickname for {}", "✔".green().bold(), port.bold().cyan());
        } else {
            println!("  {}  No nickname found for {}", "⚠".yellow(), port.bold().yellow());
        }
        return Ok(());
    }

    // ── Load config (needed by most commands) ────────────────────────────────

    let config = config::load_config()?;

    if cli.list_nicknames {
        display::print_nicknames(&config);
        return Ok(());
    }

    // ── History commands ─────────────────────────────────────────────────────

    if cli.clear_history {
        history::save_history(&history::History::default())?;
        println!("  {}  Connection history cleared.", "✔".green().bold());
        return Ok(());
    }

    if cli.history {
        let hist = history::load_history()?;
        display::print_history(&hist, cli.days, cli.relative_time);
        return Ok(());
    }

    if cli.show_recent {
        let hist = history::load_history()?;
        display::print_recent_disconnected(&hist, cli.recent_mins);
        return Ok(());
    }

    // ── Watch mode ───────────────────────────────────────────────────────────

    if cli.watch {
        let filter = if cli.all { "all" } else { &cli.filter };
        tokio::select! {
            result = watch::watch_devices(filter, cli.interval, &config) => {
                result?;
            }
            _ = tokio::signal::ctrl_c() => {
                println!();
                println!("  {}  Stopped watching. Goodbye!", "✔".green().bold());
            }
        }
        return Ok(());
    }

    // ── Default: enumerate and display ───────────────────────────────────────

    let filter = if cli.all { "all" } else { &cli.filter };
    let devices = detection::enumerate_ports(filter)?;

    // Update session: reconcile with what is currently connected
    let mut session = config::load_session().unwrap_or_default();
    let mut hist = history::load_history().unwrap_or_default();

    let current_ports: std::collections::HashSet<String> =
        devices.iter().map(|d| d.port.clone()).collect();

    // Record disconnections for ports that disappeared since last run
    let gone: Vec<String> = session
        .connected
        .keys()
        .filter(|p| !current_ports.contains(*p))
        .cloned()
        .collect();
    for port in gone {
        if let Some(sess) = session.connected.remove(&port) {
            let uptime = Utc::now()
                .signed_duration_since(sess.connected_at)
                .num_seconds();
            hist.add_event(HistoryEvent {
                port: port.clone(),
                board_name: sess.board_name.clone(),
                vid: sess.vid,
                pid: sess.pid,
                event_type: EventType::Disconnected,
                timestamp: Utc::now(),
                session_id: sess.session_id.clone(),
                uptime_secs: Some(uptime),
            });
        }
    }

    // Record connections for new ports
    for device in &devices {
        if !session.connected.contains_key(&device.port) {
            let session_id = Uuid::new_v4().to_string();
            let now = Utc::now();
            session.connected.insert(
                device.port.clone(),
                SessionDevice {
                    connected_at: now,
                    board_name: device.board_name.clone(),
                    vid: device.vid,
                    pid: device.pid,
                    session_id: session_id.clone(),
                },
            );
            hist.add_event(HistoryEvent {
                port: device.port.clone(),
                board_name: device.board_name.clone(),
                vid: device.vid,
                pid: device.pid,
                event_type: EventType::Connected,
                timestamp: now,
                session_id,
                uptime_secs: None,
            });
        }
    }

    config::save_session(&session).ok();
    history::save_history(&hist).ok();

    // ── Export mode ──────────────────────────────────────────────────────────

    if let Some(ref fmt) = cli.export {
        let content = match fmt.to_lowercase().as_str() {
            "json" => export::export_json(
                &devices,
                &config,
                &session.connected,
                cli.include_timestamps,
            )?,
            "csv" => export::export_csv(
                &devices,
                &config,
                &session.connected,
                cli.include_timestamps,
            )?,
            "table" => {
                display::print_device_list(
                    &devices,
                    &config,
                    &session.connected,
                    cli.relative_time,
                    cli.verbose,
                );
                return Ok(());
            }
            other => {
                eprintln!(
                    "  {}  Unknown export format '{}'. Use: json, csv, table",
                    "✘".red().bold(),
                    other
                );
                std::process::exit(1);
            }
        };
        export::write_output(&content, cli.output.as_deref())?;
        return Ok(());
    }

    // ── Copy mode ────────────────────────────────────────────────────────────

    if cli.copy {
        return handle_copy(&devices, &config);
    }

    // ── Default display ──────────────────────────────────────────────────────

    display::print_device_list(
        &devices,
        &config,
        &session.connected,
        cli.relative_time,
        cli.verbose,
    );

    Ok(())
}

// ─── Interactive clipboard copy ───────────────────────────────────────────────

fn handle_copy(devices: &[detection::DeviceInfo], config: &config::Config) -> Result<()> {
    if devices.is_empty() {
        println!("  {}  No devices connected.", "⚠".yellow());
        return Ok(());
    }

    println!();
    println!("{}", "  Select a port to copy to clipboard:".bold().cyan());
    println!("{}", "─".repeat(50).bright_black());
    for (i, d) in devices.iter().enumerate() {
        let nick = config
            .nicknames
            .get(&d.port)
            .map(|n| format!("  ({})", n))
            .unwrap_or_default();
        println!(
            "  [{}]  {}  —  {}{}",
            (i + 1).to_string().bold(),
            d.port.bold().cyan(),
            d.board_name,
            nick.yellow()
        );
    }
    println!();
    print!("  Enter number (1–{}): ", devices.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let idx = match input.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= devices.len() => n - 1,
        _ => {
            println!("  {}  Invalid selection.", "✘".red());
            return Ok(());
        }
    };

    let port = &devices[idx].port;

    match arboard::Clipboard::new() {
        Ok(mut cb) => match cb.set_text(port.clone()) {
            Ok(_) => println!(
                "  {}  Copied {} to clipboard!",
                "✔".green().bold(),
                port.bold().green()
            ),
            Err(e) => {
                println!("  {}  Clipboard write failed: {}", "⚠".yellow(), e);
                println!("  Port: {}", port.bold());
            }
        },
        Err(e) => {
            println!("  {}  Cannot open clipboard: {}", "⚠".yellow(), e);
            println!("  Port: {}", port.bold());
        }
    }

    Ok(())
}
