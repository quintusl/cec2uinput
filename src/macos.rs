'''
use anyhow::Result;
use hidapi::{HidApi, HidDevice};
use std::{thread, time::Duration};

use crate::Config;

pub struct HidDevice {
    device: hidapi::HidDevice,
}

impl HidDevice {
    pub fn new(config: &Config) -> Result<Self> {
        let api = HidApi::new()?;
        let device = api.open(config.vendor_id, config.product_id)?;
        Ok(Self { device })
    }

    pub fn send_key(&mut self, action: &str) -> Result<()> {
        println!("Sending key: {}", action);
        let hid_key = match action {
            "up" => 0x52,    // Up Arrow
            "down" => 0x51,  // Down Arrow
            "left" => 0x50,  // Left Arrow
            "right" => 0x4F, // Right Arrow
            "enter" => 0x28, // Return (Enter)
            "f1" => 0x3A,    // F1
            "f2" => 0x3B,    // F2
            "f3" => 0x3C,    // F3
            "f4" => 0x3D,    // F4
            _ => {
                println!("Unknown action: {}", action);
                return Ok(())
            },
        };

        let mut buf = [0u8; 8];
        buf[2] = hid_key;

        self.device.write(&buf)?;
        thread::sleep(Duration::from_millis(10));

        buf[2] = 0;
        self.device.write(&buf)?;

        Ok(())
    }
}
'''