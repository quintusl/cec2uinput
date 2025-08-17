use anyhow::Result;
use mouse_keyboard_input::VirtualDevice;
use mouse_keyboard_input::key_codes::*;
use crate::Config;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct UInputDevice {
    device: VirtualDevice,
    // counters for exponential mouse movement per axis+direction (keys like "x+", "x-", "y+", "y-")
    move_counters: HashMap<String, u8>,
    last_move: Option<Instant>,
}

impl UInputDevice {
    pub fn new(_config: &Config) -> Result<Self> {
        let device = VirtualDevice::default().map_err(|e| anyhow::anyhow!("{}", e))?;
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
                        self.device.press(mod_key).map_err(|e| anyhow::anyhow!("{}", e))?;
                        for sub in inner.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                            if let Some(k) = Self::key_from_name(sub) {
                                self.device.click(k).map_err(|e| anyhow::anyhow!("{}", e))?;
                            } else {
                                println!("Unknown action: {}", sub);
                            }
                        }
                        self.device.release(mod_key).map_err(|e| anyhow::anyhow!("{}", e))?;
                    } else {
                        println!("Unknown modifier: {}", mod_name);
                    }
                    continue;
                }
            }

            // Handle simultaneous keys with '+', e.g. "ctrl+alt+del"
            if part.contains('+') {
                let tokens: Vec<&str> = part.split('+').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
                let mut mods: Vec<u16> = Vec::new();
                let mut others: Vec<u16> = Vec::new();
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
                for mk in &mods { self.device.press(*mk).map_err(|e| anyhow::anyhow!("{}", e))?; }
                // press other keys while modifiers are held
                for k in &others { self.device.click(*k).map_err(|e| anyhow::anyhow!("{}", e))?; }
                // release modifiers
                for mk in &mods { self.device.release(*mk).map_err(|e| anyhow::anyhow!("{}", e))?; }
                continue;
            }

            // Default: single key name
            if let Some(k) = Self::key_from_name(&part) {
                self.device.click(k).map_err(|e| anyhow::anyhow!("{}", e))?;
            } else {
                println!("Unknown action: {}", part);
            }
        }

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
                self.device.move_mouse(delta, 0).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            "mouse_left" => {
                let key = "x-".to_string();
                let cnt = self.move_counters.entry(key.clone()).or_insert(0);
                if *cnt < steps.len() as u8 { *cnt += 1; }
                let idx = (*cnt as usize).saturating_sub(1).min(steps.len() - 1);
                let delta = -(steps[idx] as i32);
                self.device.move_mouse(delta, 0).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            "mouse_up" => {
                let key = "y+".to_string();
                let cnt = self.move_counters.entry(key.clone()).or_insert(0);
                if *cnt < steps.len() as u8 { *cnt += 1; }
                let idx = (*cnt as usize).saturating_sub(1).min(steps.len() - 1);
                let delta = steps[idx] as i32;
                self.device.move_mouse(0, delta).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            "mouse_down" => {
                let key = "y-".to_string();
                let cnt = self.move_counters.entry(key.clone()).or_insert(0);
                if *cnt < steps.len() as u8 { *cnt += 1; }
                let idx = (*cnt as usize).saturating_sub(1).min(steps.len() - 1);
                let delta = -(steps[idx] as i32);
                self.device.move_mouse(0, delta).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            "mouse_click_left" | "mouse_left_click" | "mouse_lclick" => {
                self.device.click(BTN_LEFT).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            "mouse_click_right" | "mouse_right_click" | "mouse_rclick" => {
                self.device.click(BTN_RIGHT).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            _ => {
                println!("Unknown mouse action: {}", action);
            }
        }

        Ok(())
    }

    // Helper: map modifier names to keys
    fn modifier_key(name: &str) -> Option<u16> {
        match name.trim() {
            "ctrl" | "control" | "lctrl" | "leftctrl" => Some(KEY_LEFTCTRL),
            "rctrl" | "rightctrl" => Some(KEY_RIGHTCTRL),
            "alt" | "lalt" | "leftalt" => Some(KEY_LEFTALT),
            "ralt" | "rightalt" => Some(KEY_RIGHTALT),
            "shift" | "lshift" | "leftshift" => Some(KEY_LEFTSHIFT),
            "rshift" | "rightshift" => Some(KEY_RIGHTSHIFT),
            "meta" | "super" | "lmeta" | "leftmeta" => Some(KEY_LEFTMETA),
            "rmeta" | "rightmeta" => Some(KEY_RIGHTMETA),
            _ => None,
        }
    }

    // Helper: map name strings to key codes (extracted from previous single-key match)
    fn key_from_name(action: &str) -> Option<u16> {
        match action.trim() {
            // Navigation
            "select" | "enter" => Some(KEY_ENTER),
            "exit" | "esc" => Some(KEY_ESC),
            "up" => Some(KEY_UP),
            "down" => Some(KEY_DOWN),
            "left" => Some(KEY_LEFT),
            "right" => Some(KEY_RIGHT),
            "home" => Some(KEY_HOME),
            "pageup" => Some(KEY_PAGEUP),
            "pagedown" => Some(KEY_PAGEDOWN),
            "end" => Some(KEY_END),
            "tab" => Some(KEY_TAB),
            "backspace" => Some(KEY_BACKSPACE),
            "delete" => Some(KEY_DELETE),
            "insert" => Some(KEY_INSERT),

            // Function keys
            "f1" => Some(KEY_F1),
            "f2" => Some(KEY_F2),
            "f3" => Some(KEY_F3),
            "f4" => Some(KEY_F4),
            "f5" => Some(KEY_F5),
            "f6" => Some(KEY_F6),
            "f7" => Some(KEY_F7),
            "f8" => Some(KEY_F8),
            "f9" => Some(KEY_F9),
            "f10" => Some(KEY_F10),
            "f11" => Some(KEY_F11),
            "f12" => Some(KEY_F12),

            // Numbers
            "0" => Some(KEY_10),
            "1" => Some(KEY_1),
            "2" => Some(KEY_2),
            "3" => Some(KEY_3),
            "4" => Some(KEY_4),
            "5" => Some(KEY_5),
            "6" => Some(KEY_6),
            "7" => Some(KEY_7),
            "8" => Some(KEY_8),
            "9" => Some(KEY_9),

            // Special keys
            "space" => Some(KEY_SPACE),
            "spacebar" => Some(KEY_SPACE),
            "dot" => Some(KEY_DOT),
            "comma" => Some(KEY_COMMA),
            "minus" => Some(KEY_MINUS),
            "equal" => Some(KEY_EQUAL),
            "slash" => Some(KEY_SLASH),
            "backslash" => Some(KEY_BACKSLASH),
            "semicolon" => Some(KEY_SEMICOLON),
            "apostrophe" => Some(KEY_APOSTROPHE),
            "leftbrace" => Some(KEY_LEFTBRACE),
            "rightbrace" => Some(KEY_RIGHTBRACE),
            "grave" => Some(KEY_GRAVE),

            // Letters
            "a" => Some(KEY_A),
            "b" => Some(KEY_B),
            "c" => Some(KEY_C),
            "d" => Some(KEY_D),
            "e" => Some(KEY_E),
            "f" => Some(KEY_F),
            "g" => Some(KEY_G),
            "h" => Some(KEY_H),
            "i" => Some(KEY_I),
            "j" => Some(KEY_J),
            "k" => Some(KEY_K),
            "l" => Some(KEY_L),
            "m" => Some(KEY_M),
            "n" => Some(KEY_N),
            "o" => Some(KEY_O),
            "p" => Some(KEY_P),
            "q" => Some(KEY_Q),
            "r" => Some(KEY_R),
            "s" => Some(KEY_S),
            "t" => Some(KEY_T),
            "u" => Some(KEY_U),
            "v" => Some(KEY_V),
            "w" => Some(KEY_W),
            "x" => Some(KEY_X),
            "y" => Some(KEY_Y),
            "z" => Some(KEY_Z),

            // Arrow keys (alternative names)
            "arrow_up" => Some(KEY_UP),
            "arrow_down" => Some(KEY_DOWN),
            "arrow_left" => Some(KEY_LEFT),
            "arrow_right" => Some(KEY_RIGHT),

            // Common aliases
            "del" => Some(KEY_DELETE),
            "ins" => Some(KEY_INSERT),
            "pgup" => Some(KEY_PAGEUP),
            "pgdown" => Some(KEY_PAGEDOWN),
            "return" => Some(KEY_ENTER),

            _ => None,
        }
    }
}
