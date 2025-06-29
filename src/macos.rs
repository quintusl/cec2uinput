
use anyhow::Result;
use hidapi::{HidApi, HidDevice};
use std::{thread, time::Duration};

use crate::Config;

pub fn run(config: &Config) -> Result<()> {
    let api = HidApi::new()?;
    let device = api.open(config.vendor_id, config.product_id)?;
    // mappings is for Linux only.
    let _mappings = config.mappings.clone();

    loop {
        let mut buf = [0u8; 64];
        let res = device.read_timeout(&mut buf[..], 50)?;
        if res > 0 {
            if let Some(action) = map_hid_to_action(&buf[..res]) {
                if let Some(hid_key) = map_action_to_hid_key(&action) {
                    send_key_event(&device, hid_key)?;
                }
            }
        }
    }
}

fn send_key_event(device: &HidDevice, hid_key: u8) -> Result<()> {
    let mut buf = [0u8; 8]; // HID keyboard report is 8 bytes
    buf[2] = hid_key; // Set the key code

    // Key down
    device.write(&buf)?;
    thread::sleep(Duration::from_millis(10)); // Small delay

    // Key up
    buf[2] = 0; // Release the key
    device.write(&buf)?;
    Ok(())
}

fn map_hid_to_action(data: &[u8]) -> Option<String> {
    // This is a placeholder. You will need to replace this with your actual
    // HID mapping logic.
    if data.len() > 0 {
        match data[0] {
            0x41 => Some("Up".to_string()),
            0x42 => Some("Down".to_string()),
            0x43 => Some("Left".to_string()),
            0x44 => Some("Right".to_string()),
            0x45 => Some("Select".to_string()),
            _ => None,
        }
    } else {
        None
    }
}

fn map_action_to_hid_key(action: &str) -> Option<u8> {
    match action {
        "Up" => Some(0x52), // HID Usage ID for Up Arrow
        "Down" => Some(0x51), // HID Usage ID for Down Arrow
        "Left" => Some(0x50), // HID Usage ID for Left Arrow
        "Right" => Some(0x4F), // HID Usage ID for Right Arrow
        "Select" => Some(0x2C), // HID Usage ID for Spacebar (common for select)
        _ => None,
    }
}
