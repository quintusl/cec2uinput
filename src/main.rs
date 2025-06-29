#[cfg(target_os = "linux")]
mod linux;



use anyhow::Result;

#[cfg(target_os = "linux")]
use cec::{Cec, CecConfiguration, CecLogMessage, CecLogLevel, CecMessage, CecOpCode, CecUserControlCode, Keypress};

use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;

#[derive(Debug, Deserialize)]
struct Config {
    device_name: String,
    vendor_id: u16,
    product_id: u16,
    mappings: HashMap<String, String>,
}

#[cfg(target_os = "linux")]
fn main() -> Result<()> {
    let config = load_config("config.yaml")?;

    let (tx, rx) = mpsc::channel();

    let cec_config = CecConfiguration {
        str_device_name: config.device_name.clone(),
        b_activate_source: true,
        callback: Some(Box::new(move |message: CecMessage| {
            if let CecMessage::KeyPress(keypress) = message {
                tx.send(keypress).unwrap();
            }
            0
        })),
        log_callback: Some(Box::new(|message: CecLogMessage| {
            match message.level {
                CecLogLevel::Error => eprintln!("CEC Error: {}", message.message),
                CecLogLevel::Warning => eprintln!("CEC Warning: {}", message.message),
                _ => println!("CEC Log: {}", message.message),
            }
        })),
        ..Default::default()
    };

    let mut cec = Cec::new(cec_config)?;
    cec.open(None)?;

    println!("CEC Initialized");

    let mut device = {
        #[cfg(target_os = "linux")]
        { linux::UInputDevice::new(&config)? }
    };

    loop {
        if let Ok(keypress) = rx.recv() {
            let key_name = match keypress {
                Keypress::UserControlCode(code) => match code {
                    CecUserControlCode::F1Blue => "f1",
                    CecUserControlCode::F2Red => "f2",
                    CecUserControlCode::F3Green => "f3",
                    CecUserControlCode::F4Yellow => "f4",
                    CecUserControlCode::Left => "left",
                    CecUserControlCode::Right => "right",
                    _ => {
                        println!("Unhandled CEC user control code: {:?}", code);
                        continue;
                    }
                },
                _ => {
                    println!("Unhandled CEC keypress: {:?}", keypress);
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

#[cfg(not(target_os = "linux"))]
fn main() -> Result<()> {
    eprintln!("This application only supports Linux.");
    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let file = File::open(path)?;
    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config)
}
