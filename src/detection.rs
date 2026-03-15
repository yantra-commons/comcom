use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Vendor IDs known to belong to Arduino / microcontroller chips.
const KNOWN_VIDS: &[(u16, &str)] = &[
    (0x2341, "Arduino LLC"),
    (0x2A03, "Arduino (clone)"),
    (0x1A86, "WCH (CH340/CH341)"),
    (0x10C4, "Silicon Labs"),
    (0x0403, "FTDI"),
    (0x0483, "STMicroelectronics"),
    (0x16C0, "PJRC (Teensy)"),
    (0x239A, "Adafruit"),
    (0x2E8A, "Raspberry Pi"),
    (0x2886, "Seeed Studio"),
    (0x1B4F, "SparkFun"),
    (0x04D8, "Microchip Technology"),
    (0x03EB, "Atmel"),
    (0x1D50, "OpenMoko / Black Magic Probe"),
    (0x0D28, "ARM mbed"),
    (0x2CCF, "Nordic Semiconductor"),
];

/// (VID, PID, human-readable board name)
const BOARD_NAMES: &[(u16, u16, &str)] = &[
    // Arduino LLC — official boards
    (0x2341, 0x0001, "Arduino Uno"),
    (0x2341, 0x0010, "Arduino Mega 2560"),
    (0x2341, 0x0036, "Arduino Leonardo"),
    (0x2341, 0x003B, "Arduino Leonardo"),
    (0x2341, 0x003F, "Arduino Mega ADK"),
    (0x2341, 0x0042, "Arduino Mega 2560 R3"),
    (0x2341, 0x0043, "Arduino Uno R3"),
    (0x2341, 0x004D, "Arduino Pro Micro"),
    (0x2341, 0x0058, "Arduino Nano Every"),
    (0x2341, 0x0070, "Arduino Due"),
    (0x2341, 0x0243, "Arduino Uno R4 Minima"),
    (0x2341, 0x0364, "Arduino Uno R4 WiFi"),
    (0x2341, 0x804D, "Arduino MKR WiFi 1010"),
    (0x2341, 0x8057, "Arduino Nano 33 BLE"),
    // WCH CH340/CH341 — clones & Nano clones
    (0x1A86, 0x7522, "Arduino Nano (CH341)"),
    (0x1A86, 0x7523, "Arduino Nano (CH340)"),
    (0x1A86, 0x55D4, "Arduino (CH343)"),
    // Silicon Labs CP210x — ESP32 etc.
    (0x10C4, 0xEA60, "ESP32 / CP210x"),
    (0x10C4, 0xEA70, "ESP32-S3 / CP210x"),
    // FTDI
    (0x0403, 0x6001, "FTDI FT232R"),
    (0x0403, 0x6010, "FTDI FT2232"),
    (0x0403, 0x6015, "FTDI FT231X"),
    // STM32
    (0x0483, 0x5740, "STM32 Virtual COM"),
    (0x0483, 0xDF11, "STM32 DFU Bootloader"),
    // Teensy
    (0x16C0, 0x0483, "Teensy"),
    // Adafruit
    (0x239A, 0x800C, "Adafruit Feather M0"),
    (0x239A, 0x8023, "Adafruit Feather M4"),
    (0x239A, 0x802B, "Adafruit Metro M4"),
    // Raspberry Pi
    (0x2E8A, 0x000A, "Raspberry Pi Pico"),
    (0x2E8A, 0x000B, "Raspberry Pi Pico W"),
    (0x2E8A, 0xF00A, "Raspberry Pi Pico (UF2)"),
    // Seeed
    (0x2886, 0x802F, "Seeed XIAO RP2040"),
    (0x2886, 0x8052, "Seeed XIAO ESP32S3"),
    // SparkFun
    (0x1B4F, 0x9205, "SparkFun Pro Micro 3.3V"),
    (0x1B4F, 0x9206, "SparkFun Pro Micro 5V"),
    // micro:bit
    (0x0D28, 0x0204, "BBC micro:bit"),
];

pub fn lookup_board_name(vid: u16, pid: u16) -> Option<String> {
    BOARD_NAMES
        .iter()
        .find(|(v, p, _)| *v == vid && *p == pid)
        .map(|(_, _, name)| name.to_string())
}

pub fn lookup_manufacturer(vid: u16) -> String {
    KNOWN_VIDS
        .iter()
        .find(|(v, _)| *v == vid)
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| format!("Unknown ({:04X})", vid))
}

pub fn is_known_mcu(vid: u16) -> bool {
    KNOWN_VIDS.iter().any(|(v, _)| *v == vid)
}

// ─── Data types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub port: String,
    pub board_name: String,
    pub manufacturer: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub serial_number: Option<String>,
    pub product: Option<String>,
    pub is_arduino: bool,
}

impl DeviceInfo {
    pub fn vid_pid_str(&self) -> String {
        match (self.vid, self.pid) {
            (Some(v), Some(p)) => format!("{:04X}:{:04X}", v, p),
            _ => "N/A".to_string(),
        }
    }
}

// ─── Enumeration ─────────────────────────────────────────────────────────────

/// Enumerate available serial ports.
///
/// `filter`:
///   - `"arduino"`  — only known MCU VIDs
///   - `"generic"`  — USB serial devices with *unknown* VIDs
///   - `"all"`      — every port type (USB + PCI + Bluetooth + unknown)
///   - anything else (e.g. `"usb"`) — USB serial only (arduino + generic)
pub fn enumerate_ports(filter: &str) -> Result<Vec<DeviceInfo>> {
    let ports = serialport::available_ports()?;
    let mut devices: Vec<DeviceInfo> = Vec::new();

    for port in ports {
        match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => {
                let vid = info.vid;
                let pid = info.pid;
                let is_arduino = is_known_mcu(vid);

                let board_name = lookup_board_name(vid, pid).unwrap_or_else(|| {
                    if is_arduino {
                        format!("Microcontroller ({:04X}:{:04X})", vid, pid)
                    } else {
                        format!("USB Serial ({:04X}:{:04X})", vid, pid)
                    }
                });

                let manufacturer = info
                    .manufacturer
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| lookup_manufacturer(vid));

                let should_include = match filter {
                    "arduino" => is_arduino,
                    "generic" => !is_arduino,
                    _ => true, // "all" or default
                };

                if should_include {
                    devices.push(DeviceInfo {
                        port: port.port_name,
                        board_name,
                        manufacturer,
                        vid: Some(vid),
                        pid: Some(pid),
                        serial_number: info.serial_number.clone(),
                        product: info.product.clone(),
                        is_arduino,
                    });
                }
            }

            non_usb => {
                // PCI / Bluetooth / Unknown — only include when filter is "all"
                if filter == "all" {
                    let board_name = match non_usb {
                        serialport::SerialPortType::PciPort => "PCI Serial Port",
                        serialport::SerialPortType::BluetoothPort => "Bluetooth Serial Port",
                        _ => "Unknown Port Type",
                    }
                    .to_string();

                    devices.push(DeviceInfo {
                        port: port.port_name,
                        board_name,
                        manufacturer: "Unknown".to_string(),
                        vid: None,
                        pid: None,
                        serial_number: None,
                        product: None,
                        is_arduino: false,
                    });
                }
            }
        }
    }

    // Natural sort so COM3 < COM10
    devices.sort_by(|a, b| {
        let a_n: Option<u32> = a.port.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok();
        let b_n: Option<u32> = b.port.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok();
        match (a_n, b_n) {
            (Some(x), Some(y)) => x.cmp(&y),
            _ => a.port.cmp(&b.port),
        }
    });

    Ok(devices)
}
