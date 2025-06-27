use anyhow::Result;
use cec_linux::{
    CecDevice, CecEvent, CecModeFollower, CecModeInitiator, CecMsg, PollFlags, PollTimeout,
};

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

    let cec = CecDevice::open("/dev/cec0")?;
    cec.set_mode(CecModeInitiator::None, CecModeFollower::Monitor)?;

    loop {
        let f = cec.poll(
            PollFlags::POLLIN | PollFlags::POLLRDNORM | PollFlags::POLLPRI,
            PollTimeout::NONE,
        )?;

        if f.intersects(PollFlags::POLLPRI) {
            let evt: CecEvent = cec.get_event()?;
            println!("evt {:?}", evt);
        }
        if f.contains(PollFlags::POLLIN | PollFlags::POLLRDNORM) {
            let msg = cec.rec()?;

            if msg.is_ok() {
                if let Err(e) = handle_cec_msg(&msg, &config, &mut device) {
                    eprintln!("Failed to handle CEC message: {}", e);
                }
            } else {
                println!("msg {:x?}", msg);
            }
        }
    }
}

fn handle_cec_msg(msg: &CecMsg, config: &Config, device: &mut uinput::Device) -> Result<()> {
    if let Some(Ok(opcode)) = msg.opcode() {
        if let cec_linux::CecOpcode::UserControlPressed = opcode {
            if let Some(param) = msg.parameters().get(0) {
                let key_name = match *param {
                    0x00 => "Select",
                    0x01 => "Up",
                    0x02 => "Down",
                    0x03 => "Left",
                    0x04 => "Right",
                    0x0d => "Enter",
                    _ => {
                        println!("Unknown key code: 0x{:02x}", param);
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
