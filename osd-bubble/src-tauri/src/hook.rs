use rdev::{Event, EventType, Key};
use std::sync::Mutex;
use std::time::Instant;
use crate::state_machine::EventCategory;

lazy_static::lazy_static! {
    static ref TRACKER: Mutex<KeyTracker> = Mutex::new(KeyTracker::new());
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedInput {
    pub keys: Vec<String>,
    pub is_mouse: bool,
    pub is_shortcut: bool,
    pub category: EventCategory,
}

pub fn parse_event(event: &Event) -> Option<ParsedInput> {
    if let Ok(mut tracker) = TRACKER.lock() {
        tracker.update(&event.event_type)
    } else {
        None
    }
}

struct KeyTracker {
    ctrl: bool,
    shift: bool,
    alt: bool,
    win: bool,

    last_main_key: Option<String>,
    repeat_count: u32,
    last_press_time: Instant,
}

impl KeyTracker {
    fn new() -> Self {
        Self {
            ctrl: false, shift: false, alt: false, win: false,
            last_main_key: None,
            repeat_count: 1,
            last_press_time: Instant::now(),
        }
    }

    fn has_modifier(&self) -> bool {
        self.ctrl || self.shift || self.alt || self.win
    }

    fn update(&mut self, event_type: &EventType) -> Option<ParsedInput> {
        // 超过 5 秒没有有效操作，自动释放所有修饰键，防止 Alt+Tab 等操作被系统吞没 release 导致的卡键
        if self.last_press_time.elapsed().as_secs() > 5 {
            self.ctrl = false;
            self.shift = false;
            self.alt = false;
            self.win = false;
            self.last_main_key = None;
        }

        match event_type {
            EventType::KeyPress(key) => {
                if self.is_modifier(*key) {
                    self.set_modifier(*key, true);
                    self.last_press_time = Instant::now(); // 更新时间防止被超时重置
                    return Some(ParsedInput {
                        keys: self.format_current(None),
                        is_mouse: false,
                        is_shortcut: true,
                        category: EventCategory::Keyboard,
                    });
                }
                
                let key_str = key_to_string(*key);
                let is_shortcut = self.has_modifier();
                
                // 连击合并逻辑
                if let Some(ref last) = self.last_main_key {
                    if last == &key_str && self.last_press_time.elapsed().as_millis() < 500 {
                        self.repeat_count += 1;
                    } else {
                        self.last_main_key = Some(key_str.clone());
                        self.repeat_count = 1;
                    }
                } else {
                    self.last_main_key = Some(key_str.clone());
                    self.repeat_count = 1;
                }
                self.last_press_time = Instant::now();

                Some(ParsedInput {
                    keys: self.format_current(Some(&key_str)),
                    is_mouse: false,
                    is_shortcut,
                    category: EventCategory::Keyboard,
                })
            }
            EventType::KeyRelease(key) => {
                if self.is_modifier(*key) {
                    self.set_modifier(*key, false);
                    self.last_main_key = None; // 释放修饰键时打断连击
                } else if let Some(ref last) = self.last_main_key {
                    let key_str = key_to_string(*key);
                    if last == &key_str {
                        // 释放主键时不清除 last_main_key，以便后续判断连击，但在超过 500ms 后会自动失效
                    }
                }
                None
            }
            EventType::ButtonPress(button) => {
                let btn_str = match button {
                    rdev::Button::Left => "LeftClick",
                    rdev::Button::Right => "RightClick",
                    rdev::Button::Middle => "MiddleClick",
                    _ => return None,
                }.to_string();
                let is_shortcut = self.has_modifier();

                if let Some(ref last) = self.last_main_key {
                    if last == &btn_str && self.last_press_time.elapsed().as_millis() < 500 {
                        self.repeat_count += 1;
                    } else {
                        self.last_main_key = Some(btn_str.clone());
                        self.repeat_count = 1;
                    }
                } else {
                    self.last_main_key = Some(btn_str.clone());
                    self.repeat_count = 1;
                }
                self.last_press_time = Instant::now();

                Some(ParsedInput {
                    keys: self.format_current(Some(&btn_str)),
                    is_mouse: true,
                    is_shortcut,
                    category: EventCategory::Mouse,
                })
            }
            EventType::Wheel { delta_x: _, delta_y } => {
                if *delta_y == 0 {
                    return None;
                }
                let btn_str = if *delta_y > 0 {
                    "ScrollUp".to_string()
                } else {
                    "ScrollDown".to_string()
                };
                let is_shortcut = self.has_modifier();

                if let Some(ref last) = self.last_main_key {
                    if last == &btn_str && self.last_press_time.elapsed().as_millis() < 500 {
                        self.repeat_count += 1;
                    } else {
                        self.last_main_key = Some(btn_str.clone());
                        self.repeat_count = 1;
                    }
                } else {
                    self.last_main_key = Some(btn_str.clone());
                    self.repeat_count = 1;
                }
                self.last_press_time = Instant::now();

                Some(ParsedInput {
                    keys: self.format_current(Some(&btn_str)),
                    is_mouse: true,
                    is_shortcut,
                    category: EventCategory::Scroll,
                })
            }
            _ => None
        }
    }

    fn is_modifier(&self, key: Key) -> bool {
        matches!(key, Key::ControlLeft | Key::ControlRight | Key::ShiftLeft | Key::ShiftRight | Key::Alt | Key::AltGr | Key::MetaLeft | Key::MetaRight)
    }

    fn set_modifier(&mut self, key: Key, pressed: bool) {
        match key {
            Key::ControlLeft | Key::ControlRight => self.ctrl = pressed,
            Key::ShiftLeft | Key::ShiftRight => self.shift = pressed,
            Key::Alt | Key::AltGr => self.alt = pressed,
            Key::MetaLeft | Key::MetaRight => self.win = pressed,
            _ => {}
        }
    }

    fn format_current(&self, main_key: Option<&str>) -> Vec<String> {
        let mut parts = Vec::new();
        
        if self.ctrl { parts.push("Ctrl".to_string()); }
        if self.win { parts.push("Win".to_string()); }
        if self.alt { parts.push("Alt".to_string()); }
        if self.shift { parts.push("Shift".to_string()); }
        
        if let Some(mk) = main_key {
            parts.push(mk.to_string());
            if self.repeat_count > 1 {
                parts.push(format!("×{}", self.repeat_count));
            }
        }
        
        parts
    }
}

fn key_to_string(key: Key) -> String {
    match key {
        Key::Escape => "Esc".to_string(),
        Key::Return => "Enter".to_string(),
        Key::Space => "Space".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Delete => "Del".to_string(),
        Key::UpArrow => "↑".to_string(),
        Key::DownArrow => "↓".to_string(),
        Key::LeftArrow => "←".to_string(),
        Key::RightArrow => "→".to_string(),
        Key::Num0 | Key::Kp0 => "0".to_string(),
        Key::Num1 | Key::Kp1 => "1".to_string(),
        Key::Num2 | Key::Kp2 => "2".to_string(),
        Key::Num3 | Key::Kp3 => "3".to_string(),
        Key::Num4 | Key::Kp4 => "4".to_string(),
        Key::Num5 | Key::Kp5 => "5".to_string(),
        Key::Num6 | Key::Kp6 => "6".to_string(),
        Key::Num7 | Key::Kp7 => "7".to_string(),
        Key::Num8 | Key::Kp8 => "8".to_string(),
        Key::Num9 | Key::Kp9 => "9".to_string(),
        Key::F1 => "F1".to_string(), Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(), Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(), Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(), Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(), Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(), Key::F12 => "F12".to_string(),
        Key::Minus | Key::KpMinus => "-".to_string(),
        Key::Equal => "=".to_string(),
        Key::LeftBracket => "[".to_string(),
        Key::RightBracket => "]".to_string(),
        Key::BackQuote => "`".to_string(),
        Key::SemiColon => ";".to_string(),
        Key::Quote => "'".to_string(),
        Key::Comma => ",".to_string(),
        Key::Dot => ".".to_string(),
        Key::Slash => "/".to_string(),
        Key::BackSlash => "\\".to_string(),
        Key::KpMultiply => "*".to_string(),
        Key::KpDivide => "/".to_string(),
        Key::KpPlus => "+".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PrintScreen => "PrtScn".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::CapsLock => "Caps".to_string(),
        _ => format!("{:?}", key).replace("Key", ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdev::{Button, Key as RdevKey};

    #[test]
    fn test_key_tracker_new() {
        let tracker = KeyTracker::new();
        assert!(!tracker.ctrl);
        assert!(!tracker.shift);
        assert!(!tracker.alt);
        assert!(!tracker.win);
        assert_eq!(tracker.repeat_count, 1);
        assert!(tracker.last_main_key.is_none());
    }

    #[test]
    fn test_is_modifier() {
        let tracker = KeyTracker::new();
        assert!(tracker.is_modifier(RdevKey::ControlLeft));
        assert!(tracker.is_modifier(RdevKey::ControlRight));
        assert!(tracker.is_modifier(RdevKey::ShiftLeft));
        assert!(tracker.is_modifier(RdevKey::ShiftRight));
        assert!(tracker.is_modifier(RdevKey::Alt));
        assert!(tracker.is_modifier(RdevKey::MetaLeft));
        // Tab and Escape are not modifiers
        assert!(!tracker.is_modifier(RdevKey::Tab));
        assert!(!tracker.is_modifier(RdevKey::Escape));
    }

    #[test]
    fn test_set_modifier() {
        let mut tracker = KeyTracker::new();
        
        tracker.set_modifier(RdevKey::ControlLeft, true);
        assert!(tracker.ctrl);
        assert!(!tracker.shift);
        assert!(!tracker.alt);
        assert!(!tracker.win);
        
        tracker.set_modifier(RdevKey::ControlLeft, false);
        assert!(!tracker.ctrl);
    }

    #[test]
    fn test_format_current_with_no_modifiers_and_no_key() {
        let tracker = KeyTracker::new();
        let result = tracker.format_current(None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_current_with_single_key() {
        let tracker = KeyTracker::new();
        let result = tracker.format_current(Some("A"));
        assert_eq!(result, vec!["A"]);
    }

    #[test]
    fn test_format_current_with_ctrl_and_key() {
        let mut tracker = KeyTracker::new();
        tracker.ctrl = true;
        let result = tracker.format_current(Some("A"));
        assert_eq!(result, vec!["Ctrl", "A"]);
    }

    #[test]
    fn test_format_current_with_repeat_count() {
        let mut tracker = KeyTracker::new();
        tracker.repeat_count = 3;
        let result = tracker.format_current(Some("A"));
        assert_eq!(result, vec!["A", "×3"]);
    }

    #[test]
    fn test_format_current_all_modifiers() {
        let mut tracker = KeyTracker::new();
        tracker.ctrl = true;
        tracker.shift = true;
        tracker.alt = true;
        tracker.win = true;
        let result = tracker.format_current(Some("A"));
        assert_eq!(result, vec!["Ctrl", "Win", "Alt", "Shift", "A"]);
    }

    #[test]
    fn test_key_to_string_arrow_keys() {
        assert_eq!(key_to_string(RdevKey::UpArrow), "↑");
        assert_eq!(key_to_string(RdevKey::DownArrow), "↓");
        assert_eq!(key_to_string(RdevKey::LeftArrow), "←");
        assert_eq!(key_to_string(RdevKey::RightArrow), "→");
    }

    #[test]
    fn test_key_to_string_special_keys() {
        assert_eq!(key_to_string(RdevKey::Escape), "Esc");
        assert_eq!(key_to_string(RdevKey::Return), "Enter");
        assert_eq!(key_to_string(RdevKey::Space), "Space");
        assert_eq!(key_to_string(RdevKey::Backspace), "Backspace");
        assert_eq!(key_to_string(RdevKey::Tab), "Tab");
        assert_eq!(key_to_string(RdevKey::Delete), "Del");
    }

    #[test]
    fn test_key_to_string_numbers() {
        assert_eq!(key_to_string(RdevKey::Num0), "0");
        assert_eq!(key_to_string(RdevKey::Num5), "5");
        assert_eq!(key_to_string(RdevKey::Num9), "9");
    }

    #[test]
    fn test_key_to_string_f_keys() {
        assert_eq!(key_to_string(RdevKey::F1), "F1");
        assert_eq!(key_to_string(RdevKey::F12), "F12");
    }

    #[test]
    fn test_button_to_string() {
        assert_eq!(button_to_string(Button::Left), "LeftClick");
        assert_eq!(button_to_string(Button::Right), "RightClick");
        assert_eq!(button_to_string(Button::Middle), "MiddleClick");
    }

    fn button_to_string(button: Button) -> String {
        match button {
            Button::Left => "LeftClick",
            Button::Right => "RightClick",
            Button::Middle => "MiddleClick",
            _ => return "Unknown".to_string(),
        }.to_string()
    }
    #[test]
    fn test_is_shortcut_detection() {
        let mut tracker = KeyTracker::new();
        
        // 普通单按键 A 不是快捷键
        let parsed = tracker.update(&EventType::KeyPress(RdevKey::KeyA)).unwrap();
        assert!(!parsed.is_shortcut);

        // 按下 Ctrl 是修饰键，标记为 shortcut
        let parsed_ctrl = tracker.update(&EventType::KeyPress(RdevKey::ControlLeft)).unwrap();
        assert!(parsed_ctrl.is_shortcut);

        // 在按住 Ctrl 时按 C，属于组合快捷键
        let parsed_combo = tracker.update(&EventType::KeyPress(RdevKey::KeyC)).unwrap();
        assert!(parsed_combo.is_shortcut);
        assert_eq!(parsed_combo.keys, vec!["Ctrl", "C"]);

        // 释放 Ctrl
        tracker.update(&EventType::KeyRelease(RdevKey::ControlLeft));

        // 再次按 C，不再是快捷键
        let parsed_single = tracker.update(&EventType::KeyPress(RdevKey::KeyC)).unwrap();
        assert!(!parsed_single.is_shortcut);
    }
}
