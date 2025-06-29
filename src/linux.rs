use anyhow::Result;
use cec::{Message, Cec};
use cec::Keypress;

use uinput::event::keyboard;

use crate::Config;

pub fn run(config: &Config) -> Result<()> {
    let mut device = uinput::default()?
        .name("cec2xdo")?
        .event(uinput::event::Keyboard::All)?
        .create()?;
    // vendor_id and product_id are for macos only.
    let _vendor_id = config.vendor_id.clone();
    let _product_id = config.product_id.clone();

    let mut cec = Cec::new().open(None)?;

    loop {
        if let Some(msg) = cec.receive() {
            if let Err(e) = handle_cec_msg(&msg, &config, &mut device) {
                eprintln!("Failed to handle CEC message: {}", e);
            }
        }
    }
}

fn handle_cec_msg(msg: &Message, config: &Config, device: &mut uinput::Device) -> Result<()> {
    if let Some(keypress) = msg.keypress() {
        let key_name = match keypress {
            Keypress::Select => "Select",
            Keypress::Up => "Up",
            Keypress::Down => "Down",
            Keypress::Left => "Left",
            Keypress::Right => "Right",
            Keypress::Enter => "Enter",
            _ => {
                println!("Unknown key code: {:?}", keypress);
                return Ok(());
            }
        };

        if let Some(action) = config.mappings.get(key_name) {
            println!("action {:?}", action);
            if let Some(key) = map_action_to_key(action) {
                device.click(&key)?;
                device.synchronize()?;
            }
        }
    }
    Ok(())
}

fn map_action_to_key(action: &str) -> Option<keyboard::Key> {
    match action {
        "Up" => Some(keyboard::Key::Up),
        "Down" => Some(keyboard::Key::Down),
        "Left" => Some(keyboard::Key::Left),
        "Right" => Some(keyboard::Key::Right),
        "Return" => Some(keyboard::Key::Enter),
        "Select" => Some(keyboard::Key::Enter),
        _ => None,
    }
}