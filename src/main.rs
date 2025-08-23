mod linux;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml_ng;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use clap::Parser;
use cec_rs::{CecConnectionCfgBuilder, CecDeviceType, CecDeviceTypeVec, CecKeypress, CecUserControlCode};
use std::ffi::CString;
use log::{debug, info, warn, error};

#[derive(Parser, Debug)]
#[command(
    author = "CEC2UInput Contributors",
    version,
    about = "CEC to uinput bridge for converting CEC remote events to keyboard/mouse input",
    long_about = "CEC2UInput bridges Consumer Electronics Control (CEC) remote control events \
to Linux uinput keyboard and mouse events. This allows you to use your TV remote \
to control applications on your Raspberry Pi or other Linux devices.

The application reads CEC events from HDMI-connected devices and translates them \
into configurable keyboard actions and mouse movements based on your configuration file.

EXAMPLES:
    cec2uinput                          # Use default config.yml with info logging
    cec2uinput -c /path/to/custom.yml   # Use custom configuration file
    cec2uinput -l debug                 # Enable debug logging for troubleshooting
    cec2uinput -q                       # Run silently (no console output)
    cec2uinput -l error -c custom.yml   # Error-only logging with custom config

CONFIGURATION:
    The configuration file (default: config.yml) defines:
    - Device name and CEC settings
    - Key mappings from CEC buttons to keyboard/mouse actions
    - Log level (can be overridden by -l flag)

    See the example config.yml for mapping syntax and available actions.

REQUIREMENTS:
    - Linux system with uinput support
    - libcec development packages installed
    - Root privileges (for uinput device creation)
    - CEC-capable HDMI hardware"
)]
struct Args {
    /// Path to the configuration file (default: config.yml)
    ///
    /// Specifies the YAML configuration file containing device settings
    /// and CEC button to keyboard/mouse action mappings
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Path to configuration file"
    )]
    config: Option<PathBuf>,

    /// Set logging verbosity level
    ///
    /// Controls the amount of diagnostic information displayed.
    /// Levels: error (minimal), warn, info (default), debug, trace (maximum)
    /// Command line setting overrides config file log_level
    #[arg(
        short,
        long,
        value_name = "LEVEL",
        help = "Log level: error, warn, info, debug, trace",
        value_parser = ["error", "warn", "info", "debug", "trace"]
    )]
    log_level: Option<String>,

    /// Suppress all console output
    ///
    /// Enables quiet mode where no messages are printed to console.
    /// Useful for running as a background service or daemon.
    /// Overrides any log level settings
    #[arg(
        short,
        long,
        help = "Run silently with no console output"
    )]
    quiet: bool,
}

#[derive(Debug, Deserialize)]
struct Config {
    device_name: String,
    #[serde(default = "default_physical_address")]
    physical_address: u16,
    #[serde(default = "default_cec_version")]
    cec_version: String,
    #[serde(default = "default_log_level")]
    log_level: String,
    mappings: HashMap<String, String>,
}

fn default_physical_address() -> u16 {
    0x1000  // Default to HDMI port 1
}

fn default_cec_version() -> String {
    "1.4".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn init_logging(level: &str, quiet: bool) -> Result<()> {
    if quiet {
        // In quiet mode, suppress all output including logs
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Off)
            .init();
        return Ok(());
    }

    let log_level = match level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => {
            // This should not happen due to value_parser, but keeping as fallback
            if !quiet {
                eprintln!("Invalid log level '{}', using 'info' instead", level);
            }
            log::LevelFilter::Info
        }
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .format_timestamp_secs()
        .init();

    Ok(())
}

#[cfg(target_os = "linux")]
fn main() -> Result<()> {
    let args = Args::parse();

    let config_path = args.config.clone().unwrap_or_else(|| {
        let default_path = PathBuf::from("config.yml");
        default_path
    });

    let file = File::open(&config_path)?;
    let config: Config = serde_yaml_ng::from_reader(file)?;

    // Determine log level: command line takes precedence, then config, then default
    let log_level = args.log_level.as_deref().unwrap_or(&config.log_level);
    init_logging(log_level, args.quiet)?;

    if args.config.is_none() {
        info!("No config file specified, using default: {}", config_path.display());
    }

    info!("Initializing CEC with device name: {}", config.device_name);

    // Create a channel for handling keypress events
    let (tx, rx) = std::sync::mpsc::channel::<CecKeypress>();

    // Configure CEC connection with enhanced Raspberry Pi CM5 compatibility
    debug!("Configuring CEC with physical address: 0x{:04x}, version: {}",
             config.physical_address, config.cec_version);

    // Try different CEC ports - CM5 has multiple CEC devices
    let cec_ports = ["/dev/cec0", "/dev/cec1", "RPI"];
    let mut cec_connection = None;

    for port in &cec_ports {
        debug!("Trying CEC port: {}", port);

        let key_press_callback = {
            let tx = tx.clone();
            Box::new(move |keypress: CecKeypress| {
                if let Err(e) = tx.send(keypress) {
                    error!("Failed to send keypress: {}", e);
                }
            })
        };

        match CecConnectionCfgBuilder::default()
            .port(CString::new(*port).unwrap())
            .device_name(config.device_name.clone())
            .device_types(CecDeviceTypeVec::new(CecDeviceType::RecordingDevice))
            .physical_address(config.physical_address)
            .monitor_only(false) // Actively participate in CEC
            .key_press_callback(key_press_callback)
            .build()
        {
            Ok(cfg) => {
                match cfg.open() {
                    Ok(conn) => {
                        info!("Successfully connected to CEC via port: {}", port);
                        cec_connection = Some(conn);
                        break;
                    }
                    Err(e) => {
                        warn!("Failed to open CEC connection on port {}: {:?}", port, e);
                        continue;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to build CEC configuration for port {}: {:?}", port, e);
                continue;
            }
        }
    }

    let _cec_connection = match cec_connection {
        Some(conn) => {
            info!("CEC connection established successfully");
            // Wait a moment for CEC to initialize properly
            std::thread::sleep(std::time::Duration::from_millis(500));
            conn
        }
        None => {
            error!("Failed to connect to any CEC port");
            error!("Common causes on Raspberry Pi CM5:");
            error!("1. Missing libcec development packages (try: sudo apt-get install libcec-dev)");
            error!("2. CEC hardware not properly detected - check 'dmesg | grep cec'");
            error!("3. Driver conflicts (try: 'sudo modprobe cec' or check /dev/cec*)");
            error!("4. Run 'cec-client -l' to check available adapters");
            error!("5. For CM5 dual HDMI, try specifying port: 'RPI:0' or 'RPI:1'");
            error!("6. Ensure user has permission to access CEC device (add to 'video' group)");
            error!("7. Check if another process is using the CEC adapter");
            error!("8. Make sure 'hdmi_ignore_cec_init=1' is NOT set in /boot/config.txt");
            error!("9. Try 'sudo systemctl stop cec' if cec service is running");
            error!("10. Verify physical address in config matches your HDMI setup");
            error!("11. Check CEC topology with 'cec-ctl --show-topology'");
            anyhow::bail!("Failed to initialize CEC on any available port");
        }
    };


    let mut device = {
        #[cfg(target_os = "linux")]
        { linux::UInputDevice::new(&config)? }
    };

    info!("CEC2UInput bridge started. Listening for CEC events...");

    loop {
        // Wait for keypress events from the callback
        if let Ok(keypress) = rx.recv() {
            // Only process initial keypress, not key repeats
            if keypress.duration.as_millis() == 0 {
                let key_code = keypress.keycode;
                let cec_event = match key_code {
                    // Navigation Controls
                    CecUserControlCode::Up => "Up",
                    CecUserControlCode::Down => "Down",
                    CecUserControlCode::Left => "Left",
                    CecUserControlCode::Right => "Right",
                    CecUserControlCode::Select => "Select",
                    CecUserControlCode::Enter => "Enter",
                    CecUserControlCode::Exit => "Exit",
                    CecUserControlCode::RightUp => "RightUp",
                    CecUserControlCode::RightDown => "RightDown",
                    CecUserControlCode::LeftUp => "LeftUp",
                    CecUserControlCode::LeftDown => "LeftDown",

                    // Menu Controls
                    CecUserControlCode::RootMenu => "RootMenu",
                    CecUserControlCode::SetupMenu => "SetupMenu",
                    CecUserControlCode::ContentsMenu => "ContentsMenu",
                    CecUserControlCode::FavoriteMenu => "FavoriteMenu",
                    CecUserControlCode::TopMenu => "TopMenu",
                    CecUserControlCode::DvdMenu => "DvdMenu",

                    // Media Controls
                    CecUserControlCode::Play => "Play",
                    CecUserControlCode::Pause => "Pause",
                    CecUserControlCode::Stop => "Stop",
                    CecUserControlCode::Record => "Record",
                    CecUserControlCode::Rewind => "Rewind",
                    CecUserControlCode::FastForward => "FastForward",
                    CecUserControlCode::Eject => "Eject",
                    CecUserControlCode::Forward => "Forward",
                    CecUserControlCode::Backward => "Backward",
                    CecUserControlCode::StopRecord => "StopRecord",
                    CecUserControlCode::PauseRecord => "PauseRecord",

                    // Audio Controls
                    CecUserControlCode::VolumeUp => "VolumeUp",
                    CecUserControlCode::VolumeDown => "VolumeDown",
                    CecUserControlCode::Mute => "Mute",
                    CecUserControlCode::SoundSelect => "SoundSelect",

                    // Power Controls
                    CecUserControlCode::Power => "Power",
                    CecUserControlCode::PowerOnFunction => "PowerOnFunction",
                    CecUserControlCode::PowerOffFunction => "PowerOffFunction",
                    CecUserControlCode::PowerToggleFunction => "PowerToggleFunction",

                    // Channel Controls
                    CecUserControlCode::ChannelUp => "ChannelUp",
                    CecUserControlCode::ChannelDown => "ChannelDown",
                    CecUserControlCode::PreviousChannel => "PreviousChannel",
                    CecUserControlCode::NextFavorite => "NextFavorite",

                    // Numeric Controls
                    CecUserControlCode::Number0 => "Number0",
                    CecUserControlCode::Number1 => "Number1",
                    CecUserControlCode::Number2 => "Number2",
                    CecUserControlCode::Number3 => "Number3",
                    CecUserControlCode::Number4 => "Number4",
                    CecUserControlCode::Number5 => "Number5",
                    CecUserControlCode::Number6 => "Number6",
                    CecUserControlCode::Number7 => "Number7",
                    CecUserControlCode::Number8 => "Number8",
                    CecUserControlCode::Number9 => "Number9",
                    CecUserControlCode::Number11 => "Number11",
                    CecUserControlCode::Number12 => "Number12",
                    CecUserControlCode::NumberEntryMode => "NumberEntryMode",
                    CecUserControlCode::Dot => "Dot",
                    CecUserControlCode::Clear => "Clear",

                    // Function Keys
                    CecUserControlCode::F1Blue => "F1Blue",
                    CecUserControlCode::F2Red => "F2Red",
                    CecUserControlCode::F3Green => "F3Green",
                    CecUserControlCode::F4Yellow => "F4Yellow",
                    CecUserControlCode::F5 => "F5",

                    // Information and Help
                    CecUserControlCode::DisplayInformation => "DisplayInformation",
                    CecUserControlCode::Help => "Help",
                    CecUserControlCode::PageUp => "PageUp",
                    CecUserControlCode::PageDown => "PageDown",
                    CecUserControlCode::InputSelect => "InputSelect",

                    // Advanced Media Functions
                    CecUserControlCode::PlayFunction => "PlayFunction",
                    CecUserControlCode::PausePlayFunction => "PausePlayFunction",
                    CecUserControlCode::RecordFunction => "RecordFunction",
                    CecUserControlCode::PauseRecordFunction => "PauseRecordFunction",
                    CecUserControlCode::StopFunction => "StopFunction",
                    CecUserControlCode::MuteFunction => "MuteFunction",
                    CecUserControlCode::RestoreVolumeFunction => "RestoreVolumeFunction",
                    CecUserControlCode::TuneFunction => "TuneFunction",
                    CecUserControlCode::SelectMediaFunction => "SelectMediaFunction",
                    CecUserControlCode::SelectAvInputFunction => "SelectAvInputFunction",
                    CecUserControlCode::SelectAudioInputFunction => "SelectAudioInputFunction",

                    // Other Controls
                    CecUserControlCode::Angle => "Angle",
                    CecUserControlCode::SubPicture => "SubPicture",
                    CecUserControlCode::VideoOnDemand => "VideoOnDemand",
                    CecUserControlCode::ElectronicProgramGuide => "ElectronicProgramGuide",
                    CecUserControlCode::TimerProgramming => "TimerProgramming",
                    CecUserControlCode::InitialConfiguration => "InitialConfiguration",
                    CecUserControlCode::SelectBroadcastType => "SelectBroadcastType",
                    CecUserControlCode::SelectSoundPresentation => "SelectSoundPresentation",
                    CecUserControlCode::Data => "Data",
                    CecUserControlCode::AnReturn => "AnReturn",
                    CecUserControlCode::AnChannelsList => "AnChannelsList",

                    // Unknown or unhandled
                    CecUserControlCode::Unknown => {
                        warn!("Unknown CEC key code received");
                        continue;
                    }
                };

                if let Some(keyboard_event) = config.mappings.get(cec_event) {
                    debug!("Mapping CEC event '{}' to input event '{}'", cec_event, keyboard_event);
                    device.send_key(keyboard_event)?;
                } else {
                    warn!("No mapping found for CEC event: {}", cec_event);
                }
            }
        }
    }
}



#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("This application is only supported on Linux.");
}
