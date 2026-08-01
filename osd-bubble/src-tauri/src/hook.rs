use rdev::{Event, EventType, Key};
use std::sync::Mutex;
use std::time::Instant;
use crate::state_machine::EventCategory;

lazy_static::lazy_static! {
    static ref TRACKER: Mutex<KeyTracker> = Mutex::new(KeyTracker::new());
}

pub struct ParsedInput {
    pub keys: Vec<String>,
    pub is_mouse: bool,
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
                        category: EventCategory::Keyboard,
                    });
                }
                
                let key_str = key_to_string(*key);
                
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
