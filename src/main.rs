mod linux;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use clap::Parser;
use cec_rs::{CecConnectionCfgBuilder, CecDeviceType, CecDeviceTypeVec, CecKeypress, CecUserControlCode};
use std::ffi::CString;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct Config {
    device_name: String,
    vendor_id: u16,
    product_id: u16,
    #[serde(default = "default_physical_address")]
    physical_address: u16,
    #[serde(default = "default_cec_version")]
    cec_version: String,
    mappings: HashMap<String, String>,
}

fn default_physical_address() -> u16 {
    0x1000  // Default to HDMI port 1
}

fn default_cec_version() -> String {
    "1.4".to_string()
}

#[cfg(target_os = "linux")]
fn main() -> Result<()> {
    let args = Args::parse();

    let config_path = args.config.unwrap_or_else(|| {
        let default_path = PathBuf::from("config.yml");
        println!("No config file specified, using default: {}", default_path.display());
        default_path
    });

    let file = File::open(&config_path)?;
    let config: Config = serde_yaml::from_reader(file)?;

    println!("Initializing CEC with device name: {}", config.device_name);

    // Create a channel for handling keypress events
    let (tx, rx) = std::sync::mpsc::channel::<CecKeypress>();

    // Configure CEC connection with enhanced Raspberry Pi CM5 compatibility
    println!("Configuring CEC with physical address: 0x{:04x}, version: {}",
             config.physical_address, config.cec_version);

    // Try different CEC ports - CM5 has multiple CEC devices
    let cec_ports = ["/dev/cec0", "/dev/cec1", "RPI"];
    let mut cec_connection = None;

    for port in &cec_ports {
        println!("Trying CEC port: {}", port);

        let key_press_callback = {
            let tx = tx.clone();
            Box::new(move |keypress: CecKeypress| {
                if let Err(e) = tx.send(keypress) {
                    eprintln!("Failed to send keypress: {}", e);
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
                        println!("Successfully connected to CEC via port: {}", port);
                        cec_connection = Some(conn);
                        break;
                    }
                    Err(e) => {
                        println!("Failed to open CEC connection on port {}: {:?}", port, e);
                        continue;
                    }
                }
            }
            Err(e) => {
                println!("Failed to build CEC configuration for port {}: {:?}", port, e);
                continue;
            }
        }
    }

    let _cec_connection = match cec_connection {
        Some(conn) => {
            println!("CEC connection established successfully");
            // Wait a moment for CEC to initialize properly
            std::thread::sleep(std::time::Duration::from_millis(500));
            conn
        }
        None => {
            eprintln!("Failed to connect to any CEC port");
            eprintln!("Common causes on Raspberry Pi CM5:");
            eprintln!("1. Missing libcec development packages (try: sudo apt-get install libcec-dev)");
            eprintln!("2. CEC hardware not properly detected - check 'dmesg | grep cec'");
            eprintln!("3. Driver conflicts (try: 'sudo modprobe cec' or check /dev/cec*)");
            eprintln!("4. Run 'cec-client -l' to check available adapters");
            eprintln!("5. For CM5 dual HDMI, try specifying port: 'RPI:0' or 'RPI:1'");
            eprintln!("6. Ensure user has permission to access CEC device (add to 'video' group)");
            eprintln!("7. Check if another process is using the CEC adapter");
            eprintln!("8. Make sure 'hdmi_ignore_cec_init=1' is NOT set in /boot/config.txt");
            eprintln!("9. Try 'sudo systemctl stop cec' if cec service is running");
            eprintln!("10. Verify physical address in config matches your HDMI setup");
            eprintln!("11. Check CEC topology with 'cec-ctl --show-topology'");
            anyhow::bail!("Failed to initialize CEC on any available port");
        }
    };


    let mut device = {
        #[cfg(target_os = "linux")]
        { linux::UInputDevice::new(&config)? }
    };

    println!("CEC2UInput bridge started. Listening for CEC events...");

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
                        println!("Unknown CEC key code received");
                        continue;
                    }
                };

                if let Some(keyboard_event) = config.mappings.get(cec_event) {
                    println!("Mapping CEC event '{}' to keyboard event '{}'", cec_event, keyboard_event);
                    device.send_key(keyboard_event)?;
                } else {
                    println!("No mapping found for CEC event: {}", cec_event);
                }
            }
        }
    }
}



#[cfg(not(target_os = "linux"))]
fn main() {
    println!("This application is only supported on Linux.");
}
