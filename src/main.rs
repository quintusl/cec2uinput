mod linux;

use anyhow::Result;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
#[cfg(target_os = "linux")]
use std::ffi::{CString};
use libcec_sys::{libcec_initialise, libcec_open, libcec_set_callbacks};
use libcec_sys::{libcec_configuration, ICECCallbacks, cec_keypress};

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

    let (tx, rx) = std::sync::mpsc::channel::<cec_keypress>();

    unsafe extern "C" fn keypress_callback(
        cb_param: *mut ::std::os::raw::c_void,
        key: *const cec_keypress
    ) {
        let tx = &*(cb_param as *const std::sync::mpsc::Sender<cec_keypress>);
        tx.send(*key).unwrap();
    }

    let tx_ptr = &tx as *const _ as *mut ::std::os::raw::c_void;

    let mut cec_config: libcec_configuration = Default::default();
    let device_name_bytes = config.device_name.as_bytes();
    let mut device_name_array: [std::os::raw::c_char; 15] = [0; 15];
    for (i, &byte) in device_name_bytes.iter().enumerate() {
        if i < 15 {
            device_name_array[i] = byte as std::os::raw::c_char;
        }
    }
    cec_config.strDeviceName = device_name_array;
    cec_config.bActivateSource = 1;
    cec_config.deviceTypes.types[0] = libcec_sys::cec_device_type_RECORDING_DEVICE;
    cec_config.iPhysicalAddress = 0x1000;
    let mut callbacks: ICECCallbacks = Default::default();
    callbacks.keyPress = Some(keypress_callback);
    cec_config.callbackParam = tx_ptr;

    println!("Initializing CEC with device name: {}", config.device_name);
    let cec = unsafe { libcec_initialise(&mut cec_config) };
    if cec.is_null() {
        eprintln!("CEC initialization failed. Common causes on Raspberry Pi:");
        eprintln!("1. Missing libcec development packages");
        eprintln!("2. CEC hardware not properly detected");
        eprintln!("3. Driver conflicts (try 'sudo modprobe cec' or check /dev/cec*)");
        eprintln!("4. Run 'cec-client -l' to check available devices");
        anyhow::bail!("Failed to initialize CEC");
    }
    unsafe { libcec_set_callbacks(cec, &mut callbacks, cec_config.callbackParam) };

    let port = CString::new("")?.into_raw(); // Use default port
    println!("Opening CEC device...");
    if unsafe { libcec_open(cec, port, 10000) } != 1 {
        eprintln!("Failed to open CEC device. This could be due to:");
        eprintln!("1. No CEC adapter found");
        eprintln!("2. Permission issues (try running as root)");
        eprintln!("3. CEC device already in use by another process");
        eprintln!("4. Incorrect device configuration");
        anyhow::bail!("Failed to open CEC device");
    }

    println!("CEC Initialized");

    let mut device = {
        #[cfg(target_os = "linux")]
        { linux::UInputDevice::new(&config)? }
    };

    loop {
        if let Ok(keypress) = rx.recv() {
            // Only process initial keypress, not key repeats
            if keypress.duration == 0 {
                let key_code = keypress.keycode;
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
            }
        }
    }
}



#[cfg(not(target_os = "linux"))]
fn main() {
    println!("This application is only supported on Linux.");
}
