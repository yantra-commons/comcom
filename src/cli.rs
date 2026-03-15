use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "com-com",
    version = "0.1.0",
    about = "Smart Arduino COM port manager",
    long_about = "com-com — Identify, monitor, and manage Arduino/microcontroller COM ports.\n\
                  Features real-time watch mode, device nicknames, and connection history."
)]
pub struct Cli {
    /// Real-time monitoring mode — shows connection/disconnection events as they happen
    #[arg(short, long)]
    pub watch: bool,

    /// Show connection history
    #[arg(long)]
    pub history: bool,

    /// Number of days of history to show (use with --history)
    #[arg(long, default_value = "1", value_name = "DAYS")]
    pub days: u32,

    /// Export in specified format: json, csv, or table
    #[arg(long, value_name = "FORMAT")]
    pub export: Option<String>,

    /// Output file for export (prints to stdout if not specified)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    /// Include full timestamp/uptime data in export output
    #[arg(long)]
    pub include_timestamps: bool,

    /// Show recently disconnected devices
    #[arg(long)]
    pub show_recent: bool,

    /// Minutes back to consider "recent" for --show-recent
    #[arg(long, default_value = "30", value_name = "MINS")]
    pub recent_mins: i64,

    /// Display relative timestamps ("2 minutes ago") instead of absolute
    #[arg(long)]
    pub relative_time: bool,

    /// Interactively select a port and copy it to clipboard
    #[arg(long)]
    pub copy: bool,

    /// Filter devices: arduino, generic, or all
    #[arg(long, value_name = "TYPE", default_value = "all")]
    pub filter: String,

    /// Clear all saved connection history
    #[arg(long)]
    pub clear_history: bool,

    /// Set a friendly nickname for a port  (e.g. --nickname COM3 "main-board")
    #[arg(long, num_args = 2, value_names = ["PORT", "NICKNAME"])]
    pub nickname: Option<Vec<String>>,

    /// Remove a previously saved nickname
    #[arg(long, value_name = "PORT")]
    pub remove_nickname: Option<String>,

    /// List all saved port nicknames
    #[arg(long)]
    pub list_nicknames: bool,

    /// Polling interval in milliseconds for --watch mode
    #[arg(long, default_value = "1000", value_name = "MS")]
    pub interval: u64,

    /// Show all serial ports including PCI and Bluetooth
    #[arg(long)]
    pub all: bool,

    /// Show verbose device information (serial number, product string)
    #[arg(short, long)]
    pub verbose: bool,
}
