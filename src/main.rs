mod linux;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
#[cfg(target_os = "linux")]
use std::ffi::{CStr, CString};
#[cfg(target_os = "linux")]
use libcec_sys::*;

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

    let (tx, rx) = std::sync::mpsc::channel();

    unsafe extern "C" fn cec_callback(
        cb_param: *mut ::std::os::raw::c_void,
        event: *const cec_event
    ) -> ::std::os::raw::c_int {
        let tx = &*(cb_param as *const std::sync::mpsc::Sender<cec_event>);
        tx.send(*event).unwrap();
        0
    }

    let tx_ptr = &tx as *const _ as *mut ::std::os::raw::c_void;

    let mut cec_config: cec_configuration = Default::default();
    cec_config.Clear();
    cec_config.strDeviceName = CString::new(config.device_name.clone())?.into_raw();
    cec_config.bActivateSource = 1;
    cec_config.callback = Some(cec_callback);
    cec_config.cbParam = tx_ptr;

    let mut cec = cec_initialise(&mut cec_config);
    if cec.is_null() {
        anyhow::bail!("Failed to initialize CEC");
    }

    let port = CString::new("")?.into_raw(); // Use default port
    if cec_open(cec, port) != 1 {
        anyhow::bail!("Failed to open CEC device");
    }

    println!("CEC Initialized");

    let mut device = {
        #[cfg(target_os = "linux")]
        { linux::UInputDevice::new(&config)? }
    };

    loop {
        if let Ok(event) = rx.recv() {
            match event.event_type {
                CEC_EVENT_TYPE_KEYPRESS => {
                    let key_code = unsafe { event.data.key.keycode };
                    let key_name = match key_code {
                        0x01 => "up",
                        0x02 => "down",
                        0x03 => "left",
                        0x04 => "right",
                        0x00 => "select", // Enter
                        0x71 => "f1", // Blue
                        0x72 => "f2", // Red
                        0x73 => "f3", // Green
                        0x74 => "f4", // Yellow
                        _ => {
                            println!("Unhandled CEC key code: 0x{:X}", key_code);
                            continue;
                        }
                    };

                    if let Some(action) = config.mappings.get(key_name) {
                        println!("Mapping '{}' to action '{}'", key_name, action);
                        device.send_key(action)?;
                    } else {
                        println!("No mapping found for key: {}", key_name);
                    }
                },
                CEC_EVENT_TYPE_COMMAND => {
                    println!("Received CEC command: {:?}", unsafe { event.data.command });
                },
                CEC_EVENT_TYPE_DEVICE_TRAFFIC => {
                    println!("Received CEC device traffic: {:?}", unsafe { event.data.traffic });
                },
                _ => {
                    println!("Unhandled CEC event type: {}", event.event_type);
                }
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() -> Result<()> {
    eprintln!("This application only supports Linux.");
    Ok(())
}

