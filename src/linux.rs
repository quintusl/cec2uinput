use anyhow::Result;
use uinput::event::keyboard;
use crate::Config;

pub struct UInputDevice {
    device: uinput::Device,
}

impl UInputDevice {
    pub fn new(config: &Config) -> Result<Self> {
        let device = uinput::default()? 
            .name(&config.device_name)?
            .vendor(config.vendor_id)
            .product(config.product_id)
            .event(uinput::event::Keyboard::All)?
            .create()?;
        Ok(Self { device })
    }

    pub fn send_key(&mut self, action: &str) -> Result<()> {
        println!("Sending key: {}", action);
        let key = match action {
            "select" => keyboard::Key::Enter,
            "enter" => keyboard::Key::Enter,
            "home" => keyboard::Key::Home,
            "blue" => keyboard::Key::F1,
            "red" => keyboard::Key::F2,
            "green" => keyboard::Key::F3,
            "yellow" => keyboard::Key::F4,
            "channel_up" => keyboard::Key::PageUp,
            "channel_down" => keyboard::Key::PageDown,
            "0" => keyboard::Key::_0,
            "1" => keyboard::Key::_1,
            "2" => keyboard::Key::_2,
            "3" => keyboard::Key::_3,
            "4" => keyboard::Key::_4,
            "5" => keyboard::Key::_5,
            "6" => keyboard::Key::_6,
            "7" => keyboard::Key::_7,
            "8" => keyboard::Key::_8,
            "9" => keyboard::Key::_9,
            "exit" => keyboard::Key::Esc,
            _ => {
                println!("Unknown action: {}", action);
                return Ok(());
            }
        };
        self.device.click(&key)?;
        self.device.synchronize()?;
        Ok(())
    }
}