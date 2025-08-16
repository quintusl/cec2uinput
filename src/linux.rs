use anyhow::Result;
use uinput::event::keyboard;
use uinput::event::{relative, controller};
use crate::Config;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct UInputDevice {
    device: uinput::Device,
    // counters for exponential mouse movement per axis+direction (keys like "x+", "x-", "y+", "y-")
    move_counters: HashMap<String, u8>,
    last_move: Option<Instant>,
}

impl UInputDevice {
    pub fn new(config: &Config) -> Result<Self> {
        let device = uinput::default()?
            .name(&config.device_name)?
            .vendor(config.vendor_id)
            .product(config.product_id)
            .event(uinput::event::Keyboard::All)?
            .event(controller::Mouse::iter_variants())?
            .event(relative::Position::iter_variants())?
            .create()?;
        Ok(Self { device, move_counters: HashMap::new(), last_move: None })
    }

    pub fn send_key(&mut self, action: &str) -> Result<()> {
        // New parser that supports sequences (comma-separated), bracketed lists, and simultaneous keys (+)
        println!("Sending key: {}", action);
        let s = action.trim();
        // Split top-level sequence items by ',' (e.g. "CTRL[c], enter")
        for part in s.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()) {
            let part = part.to_lowercase();

            // If this is a mouse mapped action, handle separately (prefix "mouse_" or exact names)
            if part.starts_with("mouse_") {
                self.send_mouse(&part)?;
                continue;
            }

            // Handle modifier with bracketed list, e.g. "alt[a,f]"
            if let Some(idx) = part.find('[') {
                if part.ends_with(']') {
                    let mod_name = part[..idx].trim();
                    let inner = &part[idx + 1..part.len() - 1];
                    // Hold modifier across the whole bracketed list (press once, send all keys, release once)
                    if let Some(mod_key) = Self::modifier_key(mod_name) {
                        self.device.press(&mod_key)?;
                        for sub in inner.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                            if let Some(k) = Self::key_from_name(sub) {
                                self.device.click(&k)?;
                            } else {
                                println!("Unknown action: {}", sub);
                            }
                        }
                        self.device.release(&mod_key)?;
                    } else {
                        println!("Unknown modifier: {}", mod_name);
                    }
                    continue;
                }
            }

            // Handle simultaneous keys with '+', e.g. "ctrl+alt+del"
            if part.contains('+') {
                let tokens: Vec<&str> = part.split('+').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
                let mut mods: Vec<keyboard::Key> = Vec::new();
                let mut others: Vec<keyboard::Key> = Vec::new();
                for tok in tokens {
                    if let Some(mk) = Self::modifier_key(tok) {
                        mods.push(mk);
                    } else if let Some(k) = Self::key_from_name(tok) {
                        others.push(k);
                    } else {
                        println!("Unknown action: {}", tok);
                    }
                }
                // press modifiers
                for mk in &mods { self.device.press(mk)?; }
                // press other keys while modifiers are held
                for k in &others { self.device.click(k)?; }
                // release modifiers
                for mk in &mods { self.device.release(mk)?; }
                continue;
            }

            // Default: single key name
            if let Some(k) = Self::key_from_name(&part) {
                self.device.click(&k)?;
            } else {
                println!("Unknown action: {}", part);
            }
        }

        // synchronize once after the sequence
        self.device.synchronize()?;
        Ok(())
    }

    // send mouse events based on mapping names like:
    // mouse_right, mouse_left, mouse_up, mouse_down, mouse_click_left, mouse_click_right
    // movement uses exponential steps: [1,10,50,100,500] and resets after 500ms idle
    fn send_mouse(&mut self, action: &str) -> Result<()> {
        // reset counters if idle > 500ms
        let now = Instant::now();
        if let Some(last) = self.last_move {
            if now.duration_since(last) > Duration::from_millis(500) {
                self.move_counters.clear();
            }
        }
        self.last_move = Some(now);

        // movement steps
        let steps = [1, 10, 50, 100, 500];

        match action {
            "mouse_right" => {
                let key = "x+".to_string();
                let cnt = self.move_counters.entry(key.clone()).or_insert(0);
                if *cnt < steps.len() as u8 { *cnt += 1; }
                let idx = (*cnt as usize).saturating_sub(1).min(steps.len() - 1);
                let delta = steps[idx] as i32;
                self.device.position(&relative::Position::X, delta)?;
            }
            "mouse_left" => {
                let key = "x-".to_string();
                let cnt = self.move_counters.entry(key.clone()).or_insert(0);
                if *cnt < steps.len() as u8 { *cnt += 1; }
                let idx = (*cnt as usize).saturating_sub(1).min(steps.len() - 1);
                let delta = -(steps[idx] as i32);
                self.device.position(&relative::Position::X, delta)?;
            }
            "mouse_down" => {
                let key = "y+".to_string();
                let cnt = self.move_counters.entry(key.clone()).or_insert(0);
                if *cnt < steps.len() as u8 { *cnt += 1; }
                let idx = (*cnt as usize).saturating_sub(1).min(steps.len() - 1);
                let delta = steps[idx] as i32;
                self.device.position(&relative::Position::Y, delta)?;
            }
            "mouse_up" => {
                let key = "y-".to_string();
                let cnt = self.move_counters.entry(key.clone()).or_insert(0);
                if *cnt < steps.len() as u8 { *cnt += 1; }
                let idx = (*cnt as usize).saturating_sub(1).min(steps.len() - 1);
                let delta = -(steps[idx] as i32);
                self.device.position(&relative::Position::Y, delta)?;
            }
            "mouse_click_left" | "mouse_left_click" | "mouse_lclick" => {
                self.device.press(&controller::Mouse::Left)?;
                self.device.release(&controller::Mouse::Left)?;
            }
            "mouse_click_right" | "mouse_right_click" | "mouse_rclick" => {
                self.device.press(&controller::Mouse::Right)?;
                self.device.release(&controller::Mouse::Right)?;
            }
            _ => {
                println!("Unknown mouse action: {}", action);
            }
        }

        self.device.synchronize()?;
        Ok(())
    }

    // Helper: map modifier names to keys
    fn modifier_key(name: &str) -> Option<keyboard::Key> {
        match name.trim() {
            "ctrl" | "control" | "lctrl" | "leftctrl" => Some(keyboard::Key::LeftCtrl),
            "rctrl" | "rightctrl" => Some(keyboard::Key::RightCtrl),
            "alt" | "lalt" | "leftalt" => Some(keyboard::Key::LeftAlt),
            "ralt" | "rightalt" => Some(keyboard::Key::RightAlt),
            "shift" | "lshift" | "leftshift" => Some(keyboard::Key::LeftShift),
            "rshift" | "rightshift" => Some(keyboard::Key::RightShift),
            "meta" | "super" | "lmeta" | "leftmeta" => Some(keyboard::Key::LeftMeta),
            "rmeta" | "rightmeta" => Some(keyboard::Key::RightMeta),
            _ => None,
        }
    }

    // Helper: map name strings to keyboard::Key (extracted from previous single-key match)
    fn key_from_name(action: &str) -> Option<keyboard::Key> {
        match action.trim() {
            // Navigation
            "select" | "enter" => Some(keyboard::Key::Enter),
            "exit" | "esc" => Some(keyboard::Key::Esc),
            "up" => Some(keyboard::Key::Up),
            "down" => Some(keyboard::Key::Down),
            "left" => Some(keyboard::Key::Left),
            "right" => Some(keyboard::Key::Right),
            "home" => Some(keyboard::Key::Home),
            "pageup" => Some(keyboard::Key::PageUp),
            "pagedown" => Some(keyboard::Key::PageDown),
            "end" => Some(keyboard::Key::End),
            "tab" => Some(keyboard::Key::Tab),
            "backspace" => Some(keyboard::Key::BackSpace),
            "delete" => Some(keyboard::Key::Delete),
            "insert" => Some(keyboard::Key::Insert),

            // Function keys
            "f1" => Some(keyboard::Key::F1),
            "f2" => Some(keyboard::Key::F2),
            "f3" => Some(keyboard::Key::F3),
            "f4" => Some(keyboard::Key::F4),
            "f5" => Some(keyboard::Key::F5),
            "f6" => Some(keyboard::Key::F6),
            "f7" => Some(keyboard::Key::F7),
            "f8" => Some(keyboard::Key::F8),
            "f9" => Some(keyboard::Key::F9),
            "f10" => Some(keyboard::Key::F10),
            "f11" => Some(keyboard::Key::F11),
            "f12" => Some(keyboard::Key::F12),

            // Numbers
            "0" => Some(keyboard::Key::_0),
            "1" => Some(keyboard::Key::_1),
            "2" => Some(keyboard::Key::_2),
            "3" => Some(keyboard::Key::_3),
            "4" => Some(keyboard::Key::_4),
            "5" => Some(keyboard::Key::_5),
            "6" => Some(keyboard::Key::_6),
            "7" => Some(keyboard::Key::_7),
            "8" => Some(keyboard::Key::_8),
            "9" => Some(keyboard::Key::_9),

            // Special keys
            "space" => Some(keyboard::Key::Space),
            "spacebar" => Some(keyboard::Key::Space),
            "dot" => Some(keyboard::Key::Dot),
            "comma" => Some(keyboard::Key::Comma),
            "minus" => Some(keyboard::Key::Minus),
            "equal" => Some(keyboard::Key::Equal),
            "slash" => Some(keyboard::Key::Slash),
            "backslash" => Some(keyboard::Key::BackSlash),
            "semicolon" => Some(keyboard::Key::SemiColon),
            "apostrophe" => Some(keyboard::Key::Apostrophe),
            "leftbrace" => Some(keyboard::Key::LeftBrace),
            "rightbrace" => Some(keyboard::Key::RightBrace),
            "grave" => Some(keyboard::Key::Grave),

            // Letters
            "a" => Some(keyboard::Key::A),
            "b" => Some(keyboard::Key::B),
            "c" => Some(keyboard::Key::C),
            "d" => Some(keyboard::Key::D),
            "e" => Some(keyboard::Key::E),
            "f" => Some(keyboard::Key::F),
            "g" => Some(keyboard::Key::G),
            "h" => Some(keyboard::Key::H),
            "i" => Some(keyboard::Key::I),
            "j" => Some(keyboard::Key::J),
            "k" => Some(keyboard::Key::K),
            "l" => Some(keyboard::Key::L),
            "m" => Some(keyboard::Key::M),
            "n" => Some(keyboard::Key::N),
            "o" => Some(keyboard::Key::O),
            "p" => Some(keyboard::Key::P),
            "q" => Some(keyboard::Key::Q),
            "r" => Some(keyboard::Key::R),
            "s" => Some(keyboard::Key::S),
            "t" => Some(keyboard::Key::T),
            "u" => Some(keyboard::Key::U),
            "v" => Some(keyboard::Key::V),
            "w" => Some(keyboard::Key::W),
            "x" => Some(keyboard::Key::X),
            "y" => Some(keyboard::Key::Y),
            "z" => Some(keyboard::Key::Z),

            // Arrow keys (alternative names)
            "arrow_up" => Some(keyboard::Key::Up),
            "arrow_down" => Some(keyboard::Key::Down),
            "arrow_left" => Some(keyboard::Key::Left),
            "arrow_right" => Some(keyboard::Key::Right),

            // Common aliases
            "del" => Some(keyboard::Key::Delete),
            "ins" => Some(keyboard::Key::Insert),
            "pgup" => Some(keyboard::Key::PageUp),
            "pgdown" => Some(keyboard::Key::PageDown),
            "return" => Some(keyboard::Key::Enter),

            _ => None,
        }
    }
}
