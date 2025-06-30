#[cfg(target_os = "linux")]
use anyhow::Result;
#[cfg(target_os = "linux")]
use uinput::event::keyboard;
#[cfg(target_os = "linux")]
use crate::Config;

#[cfg(target_os = "linux")]
pub struct UInputDevice {
    device: uinput::Device,
}

#[cfg(target_os = "linux")]
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
            "up" => keyboard::Key::Up,
            "down" => keyboard::Key::Down,
            "left" => keyboard::Key::Left,
            "right" => keyboard::Key::Right,
            "enter" => keyboard::Key::Enter,
            "f1" => keyboard::Key::F1,
            "f2" => keyboard::Key::F2,
            "f3" => keyboard::Key::F3,
            "f4" => keyboard::Key::F4,
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
