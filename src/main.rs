mod linux;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use cec_rs::{CecConnectionCfgBuilder, CecDeviceType, CecDeviceTypeVec, CecKeypress, CecUserControlCode};
use std::ffi::CString;

#[derive(Debug, Deserialize)]
struct Config {
    device_name: String,
    vendor_id: u16,
    product_id: u16,
    mappings: HashMap<String, String>,
}

#[cfg(target_os = "linux")]
fn main() -> Result<()> {
    let file = File::open("config.yaml")?;
    let config: Config = serde_yaml::from_reader(file)?;

    println!("Initializing CEC with device name: {}", config.device_name);

    // Create a channel for handling keypress events
    let (tx, rx) = std::sync::mpsc::channel::<CecKeypress>();

    // Define keypress callback
    let key_press_callback = {
        let tx = tx.clone();
        Box::new(move |keypress: CecKeypress| {
            if let Err(e) = tx.send(keypress) {
                eprintln!("Failed to send keypress: {}", e);
            }
        })
    };

    // Configure CEC connection with improved Raspberry Pi compatibility
    let cec_config = CecConnectionCfgBuilder::default()
        .port(CString::new("RPI").unwrap()) // Use Raspberry Pi CEC port
        .device_name(config.device_name.clone())
        .device_types(CecDeviceTypeVec::new(CecDeviceType::RecordingDevice))
        .key_press_callback(key_press_callback)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build CEC configuration: {}", e))?;

    // Initialize CEC connection with better error handling for Raspberry Pi
    let _cec_connection = match cec_config.open() {
        Ok(connection) => {
            println!("CEC connection established successfully");
            connection
        }
        Err(e) => {
            eprintln!("CEC initialization failed: {:?}", e);
            eprintln!("Common causes on Raspberry Pi:");
            eprintln!("1. Missing libcec development packages (try: sudo apt-get install libcec-dev)");
            eprintln!("2. CEC hardware not properly detected");
            eprintln!("3. Driver conflicts (try: 'sudo modprobe cec' or check /dev/cec*)");
            eprintln!("4. Run 'cec-client -l' to check available adapters");
            eprintln!("5. Ensure user has permission to access CEC device");
            eprintln!("6. Check if another process is using the CEC adapter");
            eprintln!("7. Make sure 'hdmi_ignore_cec_init=1' is NOT set in /boot/config.txt");
            eprintln!("8. Try 'sudo systemctl stop cec' if cec service is running");
            anyhow::bail!("Failed to initialize CEC: {:?}", e);
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
                let key_name = match key_code {
                    CecUserControlCode::Up => "up",
                    CecUserControlCode::Down => "down", 
                    CecUserControlCode::Left => "left",
                    CecUserControlCode::Right => "right",
                    CecUserControlCode::Select => "select", // Enter
                    CecUserControlCode::F1Blue => "f1", // Blue
                    CecUserControlCode::F2Red => "f2", // Red
                    CecUserControlCode::F3Green => "f3", // Green
                    CecUserControlCode::F4Yellow => "f4", // Yellow
                    _ => {
                        println!("Unhandled CEC key code: {:?}", key_code);
                        continue;
                    }
                };

                if let Some(action) = config.mappings.get(key_name) {
                    println!("Mapping '{}' to action '{}'", key_name, action);
                    device.send_key(action)?;
                } else {
                    println!("No mapping found for key: {}", key_name);
                }
            }
        }
    }
}



#[cfg(not(target_os = "linux"))]
fn main() {
    println!("This application is only supported on Linux.");
}
