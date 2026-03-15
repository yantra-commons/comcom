# com-com

A smart CLI tool for identifying, monitoring, and managing Arduino and microcontroller COM/serial ports.

## Features

- Detect connected Arduino boards and other microcontrollers by VID/PID
- Real-time watch mode — see devices connect and disconnect live
- Assign friendly nicknames to ports
- View full connection history with timestamps and uptime
- Export device list to JSON, CSV, or table format
- Copy a port name to clipboard interactively

## Supported boards

Arduino (Uno, Mega, Leonardo, Nano, MKR, Nano 33 BLE, Uno R4...),
ESP32 / CP210x, FTDI, STM32, Teensy, Adafruit Feather/Metro,
Raspberry Pi Pico, Seeed XIAO, SparkFun Pro Micro, BBC micro:bit,
WCH CH340/CH341 clones, and more.

## Installation

### From source (requires [Rust](https://rustup.rs))

```sh
cargo install --path .
```

Or install directly from crates.io once published:

```sh
cargo install com-com
```

### Windows — linker setup

Rust on Windows needs a linker. Pick one:

**Option A — MSVC (recommended):**
Install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) with the "Desktop development with C++" workload, then:
```sh
rustup default stable-x86_64-pc-windows-msvc
```

**Option B — GNU via MSYS2:**
1. Install [MSYS2](https://www.msys2.org/)
2. In an MSYS2 MinGW64 terminal: `pacman -S mingw-w64-x86_64-gcc`
3. Add `C:\msys64\mingw64\bin` to your system PATH
4. `rustup default stable-x86_64-pc-windows-gnu`

If MSYS2 is not on your PATH, uncomment the linker block in `.cargo/config.toml` and point it at your GCC installation.

### Linux

No extra steps — `cargo build` works out of the box.

You may need `libudev` for the `serialport` crate:
```sh
# Debian/Ubuntu
sudo apt install libudev-dev

# Fedora/RHEL
sudo dnf install systemd-devel
```

### macOS

No extra steps — `cargo build` works out of the box.

## Usage

```sh
# List connected Arduino/MCU devices
com-com

# Show all serial ports (including PCI, Bluetooth, unknown)
com-com --all

# Real-time watch mode (Ctrl+C to stop)
com-com --watch

# Show connection history (last 24 hours by default)
com-com --history

# Show history for the last 7 days
com-com --history --days 7

# Show devices disconnected in the last 30 minutes
com-com --show-recent

# Verbose output (serial number, product string)
com-com --verbose

# Assign a nickname to a port
com-com --nickname COM3 "main-board"       # Windows
com-com --nickname /dev/ttyUSB0 "sensor"  # Linux/macOS

# Remove a nickname
com-com --remove-nickname COM3

# List all saved nicknames
com-com --list-nicknames

# Interactively copy a port name to clipboard
com-com --copy

# Export to JSON or CSV
com-com --export json
com-com --export csv
com-com --export json --output devices.json
com-com --export csv  --output devices.csv --include-timestamps

# Filter to only known MCU boards or only unknown USB serial devices
com-com --filter arduino
com-com --filter generic

# Adjust watch polling interval (milliseconds)
com-com --watch --interval 500

# Clear all saved connection history
com-com --clear-history
```

## Data storage

All persistent data lives in your OS application data directory:

| OS      | Path                                           |
|---------|------------------------------------------------|
| Windows | `%APPDATA%\com-com\`                           |
| macOS   | `~/Library/Application Support/com-com/`       |
| Linux   | `~/.local/share/com-com/` (or `$XDG_DATA_HOME`) |

Files:
- `config.json` — saved nicknames
- `session.json` — current session connection timestamps
- `history.json` — full connection/disconnection event log

## Contributing

Pull requests are welcome. To get started:

```sh
git clone https://github.com/venkata-sai-vishwanath-robo/com-com
cd com-com
cargo build
cargo run -- --help
```

To add board support, edit the `KNOWN_VIDS` and `BOARD_NAMES` tables in `src/detection.rs`.

## License

MIT — see [LICENSE](LICENSE).
