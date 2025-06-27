use anyhow::Result;
use crate::Config;
use hidapi::{HidApi, HidDevice};
use std::{thread, time::Duration};

#[repr(C)]
struct CecMessage {
    opcode: u8,
    num_params: u8,
    params: [u8; 16],
}

#[link(name = "cec_shim")]
extern "C" {
    fn initialize_cec();
    fn get_cec_message(msg: *mut CecMessage) -> i32;
}

pub fn run(config: &Config) -> Result<()> {
    unsafe {
        initialize_cec();
    }

    let api = HidApi::new()?;
    // TODO: Make Vendor ID and Product ID configurable
    let device = api.open(config.vendor_id, config.product_id)?;

    loop {
        let mut msg = CecMessage { opcode: 0, num_params: 0, params: [0; 16] };
        let result = unsafe { get_cec_message(&mut msg) };

        if result == 1 {
            // Message received
            println!("Received CEC message: opcode={:x}, num_params={}", msg.opcode, msg.num_params);
            if msg.opcode == 0x44 && msg.num_params > 0 { // User Control Pressed
                let key_code = msg.params[0];
                if let Some(action) = map_cec_key_to_action(key_code) {
                    if let Some(hid_key) = map_action_to_hid_key(&action) {
                        // Simulate key press using hidapi
                        send_key_event(&device, hid_key)?;
                    }
                }
            }
        } else if result == 0 {
            // No message received, sleep for a bit to avoid busy-looping
            thread::sleep(Duration::from_millis(50));
        } else {
            // Error
            eprintln!("Error getting CEC message");
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

fn map_cec_key_to_action(cec_key_code: u8) -> Option<String> {
    match cec_key_code {
        0x00 => Some("Select".to_string()),
        0x01 => Some("Up".to_string()),
        0x02 => Some("Down".to_string()),
        0x03 => Some("Left".to_string()),
        0x04 => Some("Right".to_string()),
        0x0d => Some("Enter".to_string()),
        _ => None,
    }
}

fn map_action_to_hid_key(action: &str) -> Option<u8> {
    match action {
        "Up" => Some(0x52), // HID Usage ID for Up Arrow
        "Down" => Some(0x51), // HID Usage ID for Down Arrow
        "Left" => Some(0x50), // HID Usage ID for Left Arrow
        "Right" => Some(0x4F), // HID Usage ID for Right Arrow
        "Enter" => Some(0x28), // HID Usage ID for Enter
        "Select" => Some(0x2C), // HID Usage ID for Spacebar (common for select)
        _ => None,
    }
}
