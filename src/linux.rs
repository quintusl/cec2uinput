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
            // Navigation
            "select" => keyboard::Key::Enter,
            "enter" => keyboard::Key::Enter,
            "exit" => keyboard::Key::Esc,
            "esc" => keyboard::Key::Esc,
            "up" => keyboard::Key::Up,
            "down" => keyboard::Key::Down,
            "left" => keyboard::Key::Left,
            "right" => keyboard::Key::Right,
            "home" => keyboard::Key::Home,
            "pageup" => keyboard::Key::PageUp,
            "pagedown" => keyboard::Key::PageDown,
            "end" => keyboard::Key::End,
            "tab" => keyboard::Key::Tab,
            "backspace" => keyboard::Key::BackSpace,
            "delete" => keyboard::Key::Delete,
            "insert" => keyboard::Key::Insert,
            
            // Function keys
            "f1" => keyboard::Key::F1,
            "f2" => keyboard::Key::F2,
            "f3" => keyboard::Key::F3,
            "f4" => keyboard::Key::F4,
            "f5" => keyboard::Key::F5,
            "f6" => keyboard::Key::F6,
            "f7" => keyboard::Key::F7,
            "f8" => keyboard::Key::F8,
            "f9" => keyboard::Key::F9,
            "f10" => keyboard::Key::F10,
            "f11" => keyboard::Key::F11,
            "f12" => keyboard::Key::F12,
            
            // Numbers
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
            
            // Special keys
            "space" => keyboard::Key::Space,
            "dot" => keyboard::Key::Dot,
            "comma" => keyboard::Key::Comma,
            "minus" => keyboard::Key::Minus,
            "equal" => keyboard::Key::Equal,
            "slash" => keyboard::Key::Slash,
            "backslash" => keyboard::Key::BackSlash,
            "semicolon" => keyboard::Key::SemiColon,
            "apostrophe" => keyboard::Key::Apostrophe,
            "leftbrace" => keyboard::Key::LeftBrace,
            "rightbrace" => keyboard::Key::RightBrace,
            "grave" => keyboard::Key::Grave,
            
            // Letters (for advanced mappings)
            "a" => keyboard::Key::A,
            "b" => keyboard::Key::B,
            "c" => keyboard::Key::C,
            "d" => keyboard::Key::D,
            "e" => keyboard::Key::E,
            "f" => keyboard::Key::F,
            "g" => keyboard::Key::G,
            "h" => keyboard::Key::H,
            "i" => keyboard::Key::I,
            "j" => keyboard::Key::J,
            "k" => keyboard::Key::K,
            "l" => keyboard::Key::L,
            "m" => keyboard::Key::M,
            "n" => keyboard::Key::N,
            "o" => keyboard::Key::O,
            "p" => keyboard::Key::P,
            "q" => keyboard::Key::Q,
            "r" => keyboard::Key::R,
            "s" => keyboard::Key::S,
            "t" => keyboard::Key::T,
            "u" => keyboard::Key::U,
            "v" => keyboard::Key::V,
            "w" => keyboard::Key::W,
            "x" => keyboard::Key::X,
            "y" => keyboard::Key::Y,
            "z" => keyboard::Key::Z,
            
            // Arrow keys (alternative names)
            "arrow_up" => keyboard::Key::Up,
            "arrow_down" => keyboard::Key::Down,
            "arrow_left" => keyboard::Key::Left,
            "arrow_right" => keyboard::Key::Right,
            
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
