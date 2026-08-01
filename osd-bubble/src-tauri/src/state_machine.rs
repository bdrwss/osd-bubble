use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BubbleState {
    Idle,
    Visible { start: Instant },
    FadingOut { start: Instant },
}

/// 事件分类，用于过滤显示
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventCategory {
    Keyboard,
    Mouse,
    Scroll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStyle {
    pub bg_color: String,
    pub bg_opacity: f32,
    pub text_color: String,
    pub border_color: String,
    pub border_width: f32,
    pub radius: f32,
    pub shadow_color: String,
}

impl CustomStyle {
    pub fn new() -> Self {
        Self {
            bg_color: "#000000".to_string(),
            bg_opacity: 0.7,
            text_color: "#ffffff".to_string(),
            border_color: "#000000".to_string(),
            border_width: 0.0,
            radius: 8.0,
            shadow_color: "#000000".to_string(),
        }
    }
}

impl Default for CustomStyle {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StateMachine {
    pub state: BubbleState,
    pub visible_duration: Duration,
    pub fade_duration: Duration,
    pub quadrant: u8,
    pub bubble_style: String,
    pub exclude_apps: Vec<String>,
    pub custom_style: CustomStyle,
    // V1.0 新增字段
    pub enabled: bool,
    pub show_keyboard: bool,
    pub show_mouse: bool,
    pub show_scroll: bool,
    pub opacity: f32,
    pub theme: String,
    pub scale: f32,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: BubbleState::Idle,
            visible_duration: Duration::from_millis(1200),
            fade_duration: Duration::from_millis(280),
            quadrant: 3, // 默认右下
            bubble_style: "default".to_string(),
            exclude_apps: Vec::new(),
            custom_style: CustomStyle::new(),
            enabled: true,
            show_keyboard: true,
            show_mouse: true,
            show_scroll: true,
            opacity: 0.85,
            theme: "dark".to_string(),
            scale: 1.0,
        }
    }

    /// 根据事件分类判断是否应该显示气泡
    pub fn should_show_event(&self, category: EventCategory) -> bool {
        if !self.enabled {
            return false;
        }
        match category {
            EventCategory::Keyboard => self.show_keyboard,
            EventCategory::Mouse => self.show_mouse,
            EventCategory::Scroll => self.show_scroll,
        }
    }

    pub fn on_key_press(&mut self) {
        self.state = BubbleState::Visible { start: Instant::now() };
    }

    /// Returns the current opacity (0.0 to 1.0) and whether a redraw is needed
    /// 返回的 alpha 已乘以全局 opacity
    pub fn tick(&mut self) -> (f32, bool) {
        let now = Instant::now();
        let (raw_alpha, needs_redraw) = match self.state {
            BubbleState::Idle => (0.0, false),
            BubbleState::Visible { start } => {
                if now.duration_since(start) >= self.visible_duration {
                    self.state = BubbleState::FadingOut { start: now };
                    (1.0, true)
                } else {
                    (1.0, false)
                }
            }
            BubbleState::FadingOut { start } => {
                let elapsed = now.duration_since(start);
                if elapsed >= self.fade_duration {
                    self.state = BubbleState::Idle;
                    (0.0, true)
                } else {
                    let progress = elapsed.as_secs_f32() / self.fade_duration.as_secs_f32();
                    // ease-out fade
                    let alpha = 1.0 - progress;
                    (alpha, true)
                }
            }
        };
        (raw_alpha * self.opacity, needs_redraw)
    }

    /// 应用预设配置
    pub fn apply_preset(&mut self, preset: &str) {
        match preset {
            "classroom" => {
                self.visible_duration = Duration::from_millis(2000);
                self.opacity = 1.0;
                self.bubble_style = "3d_key".to_string();
                self.scale = 1.5;
            }
            "recording" => {
                self.visible_duration = Duration::from_millis(1500);
                self.opacity = 0.85;
                self.bubble_style = "default".to_string();
                self.scale = 1.0;
            }
            "streaming" => {
                self.visible_duration = Duration::from_millis(800);
                self.opacity = 0.6;
                self.bubble_style = "retro_terminal".to_string();
                self.scale = 0.8;
            }
            _ => {}
        }
    }

    /// 重置所有字段为默认值
    pub fn reset_to_defaults(&mut self) {
        *self = Self::new();
    }
}
