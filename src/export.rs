use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;

use crate::config::{Config, SessionDevice};
use crate::detection::DeviceInfo;
use crate::history::{format_local, format_uptime};

// ─── Export record ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ExportDevice {
    port: String,
    board_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,
    vid: Option<String>,
    pid: Option<String>,
    vid_pid: String,
    manufacturer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    serial_number: Option<String>,
    is_arduino: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    connected_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uptime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uptime_seconds: Option<i64>,
}

fn build_records(
    devices: &[DeviceInfo],
    config: &Config,
    session: &HashMap<String, SessionDevice>,
    include_timestamps: bool,
) -> Vec<ExportDevice> {
    let now = Utc::now();

    devices
        .iter()
        .map(|d| {
            let nickname = config.nicknames.get(&d.port).cloned();

            let (connected_at, uptime, uptime_seconds) = if include_timestamps {
                if let Some(sess) = session.get(&d.port) {
                    let secs = now.signed_duration_since(sess.connected_at).num_seconds();
                    (
                        Some(format_local(sess.connected_at)),
                        Some(format_uptime(secs)),
                        Some(secs),
                    )
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

            ExportDevice {
                port: d.port.clone(),
                board_name: d.board_name.clone(),
                nickname,
                vid: d.vid.map(|v| format!("{:04X}", v)),
                pid: d.pid.map(|p| format!("{:04X}", p)),
                vid_pid: d.vid_pid_str(),
                manufacturer: d.manufacturer.clone(),
                serial_number: d.serial_number.clone(),
                is_arduino: d.is_arduino,
                connected_at,
                uptime,
                uptime_seconds,
            }
        })
        .collect()
}

// ─── JSON ─────────────────────────────────────────────────────────────────────

pub fn export_json(
    devices: &[DeviceInfo],
    config: &Config,
    session: &HashMap<String, SessionDevice>,
    include_timestamps: bool,
) -> Result<String> {
    let records = build_records(devices, config, session, include_timestamps);
    Ok(serde_json::to_string_pretty(&records)?)
}

// ─── CSV ──────────────────────────────────────────────────────────────────────

pub fn export_csv(
    devices: &[DeviceInfo],
    config: &Config,
    session: &HashMap<String, SessionDevice>,
    include_timestamps: bool,
) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);

    // Header
    if include_timestamps {
        wtr.write_record(&[
            "Port", "Board", "Nickname", "VID:PID", "Manufacturer",
            "Serial", "Is Arduino", "Connected At", "Uptime", "Uptime (s)",
        ])?;
    } else {
        wtr.write_record(&[
            "Port", "Board", "Nickname", "VID:PID", "Manufacturer",
            "Serial", "Is Arduino",
        ])?;
    }

    let records = build_records(devices, config, session, include_timestamps);
    for r in &records {
        let nick  = r.nickname.clone().unwrap_or_default();
        let serial = r.serial_number.clone().unwrap_or_default();
        let is_a   = if r.is_arduino { "true" } else { "false" };

        if include_timestamps {
            wtr.write_record(&[
                &r.port,
                &r.board_name,
                &nick,
                &r.vid_pid,
                &r.manufacturer,
                &serial,
                is_a,
                r.connected_at.as_deref().unwrap_or(""),
                r.uptime.as_deref().unwrap_or(""),
                &r.uptime_seconds.map(|s| s.to_string()).unwrap_or_default(),
            ])?;
        } else {
            wtr.write_record(&[
                &r.port,
                &r.board_name,
                &nick,
                &r.vid_pid,
                &r.manufacturer,
                &serial,
                is_a,
            ])?;
        }
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| anyhow!("CSV flush error: {}", e))?;
    Ok(String::from_utf8(bytes)?)
}

// ─── Output ───────────────────────────────────────────────────────────────────

pub fn write_output(content: &str, path: Option<&str>) -> Result<()> {
    match path {
        Some(p) => {
            std::fs::write(p, content)?;
            println!("Exported to: {}", p);
        }
        None => println!("{}", content),
    }
    Ok(())
}
