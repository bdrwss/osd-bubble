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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_initial_state() {
        let sm = StateMachine::new();
        assert_eq!(sm.state, BubbleState::Idle);
        assert_eq!(sm.visible_duration, Duration::from_millis(1200));
        assert_eq!(sm.fade_duration, Duration::from_millis(280));
        assert!(sm.enabled);
        assert!(sm.show_keyboard);
        assert!(sm.show_mouse);
        assert!(sm.show_scroll);
    }

    #[test]
    fn test_on_key_press_transitions_to_visible() {
        let mut sm = StateMachine::new();
        sm.state = BubbleState::Idle;
        
        sm.on_key_press();
        
        assert!(matches!(sm.state, BubbleState::Visible { .. }));
    }

    #[test]
    fn test_tick_idle_returns_zero_opacity() {
        let mut sm = StateMachine::new();
        sm.state = BubbleState::Idle;
        
        let (opacity, needs_redraw) = sm.tick();
        assert_eq!(opacity, 0.0);
        assert!(!needs_redraw);
    }

    #[test]
    fn test_tick_visible_stays_visible() {
        let mut sm = StateMachine::new();
        sm.state = BubbleState::Visible { start: Instant::now() };
        
        let (opacity, needs_redraw) = sm.tick();
        assert_eq!(opacity, 0.85); // default opacity
        assert!(!needs_redraw);
    }

    #[test]
    fn test_tick_visible_transitions_to_fading_out() {
        let mut sm = StateMachine::new();
        sm.visible_duration = Duration::from_millis(100);
        sm.state = BubbleState::Visible { start: Instant::now() - Duration::from_millis(150) };
        
        let (opacity, needs_redraw) = sm.tick();
        assert_eq!(opacity, 0.85);
        assert!(needs_redraw);
        assert!(matches!(sm.state, BubbleState::FadingOut { .. }));
    }

    #[test]
    fn test_tick_fading_out_reaches_idle() {
        let mut sm = StateMachine::new();
        sm.fade_duration = Duration::from_millis(100);
        sm.state = BubbleState::FadingOut { start: Instant::now() - Duration::from_millis(150) };
        
        let (opacity, needs_redraw) = sm.tick();
        assert_eq!(opacity, 0.0);
        assert!(needs_redraw);
        assert_eq!(sm.state, BubbleState::Idle);
    }

    #[test]
    fn test_tick_fading_out_calculates_alpha() {
        let mut sm = StateMachine::new();
        sm.fade_duration = Duration::from_millis(1000);
        sm.opacity = 1.0; // Override default to make calculation easier
        sm.state = BubbleState::FadingOut { start: Instant::now() - Duration::from_millis(500) };
        
        let (alpha, needs_redraw) = sm.tick();
        assert!(alpha > 0.0 && alpha < 1.0);
        assert!(needs_redraw);
    }

    #[test]
    fn test_should_show_event_respects_enabled_flag() {
        let mut sm = StateMachine::new();
        sm.enabled = false;
        
        assert!(!sm.should_show_event(EventCategory::Keyboard));
        assert!(!sm.should_show_event(EventCategory::Mouse));
        assert!(!sm.should_show_event(EventCategory::Scroll));
    }

    #[test]
    fn test_should_show_event_respects_category_flags() {
        let mut sm = StateMachine::new();
        sm.show_keyboard = false;
        sm.show_mouse = false;
        sm.show_scroll = false;
        
        assert!(!sm.should_show_event(EventCategory::Keyboard));
        assert!(!sm.should_show_event(EventCategory::Mouse));
        assert!(!sm.should_show_event(EventCategory::Scroll));
        
        sm.show_keyboard = true;
        assert!(sm.should_show_event(EventCategory::Keyboard));
    }

    #[test]
    fn test_apply_preset_classroom() {
        let mut sm = StateMachine::new();
        sm.apply_preset("classroom");
        
        assert_eq!(sm.visible_duration, Duration::from_millis(2000));
        assert_eq!(sm.opacity, 1.0);
        assert_eq!(sm.bubble_style, "3d_key");
        assert_eq!(sm.scale, 1.5);
    }

    #[test]
    fn test_apply_preset_unknown_ignores() {
        let mut sm = StateMachine::new();
        let original_duration = sm.visible_duration;
        let original_style = sm.bubble_style.clone();
        
        sm.apply_preset("unknown_preset");
        
        assert_eq!(sm.visible_duration, original_duration);
        assert_eq!(sm.bubble_style, original_style);
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut sm = StateMachine::new();
        sm.visible_duration = Duration::from_millis(5000);
        sm.enabled = false;
        sm.apply_preset("classroom");
        
        sm.reset_to_defaults();
        
        assert_eq!(sm.state, BubbleState::Idle);
        assert_eq!(sm.visible_duration, Duration::from_millis(1200));
        assert!(sm.enabled);
        assert_eq!(sm.bubble_style, "default");
    }

    #[test]
    fn test_full_lifecycle() {
        let mut sm = StateMachine::new();
        
        // Initial idle state
        let (alpha, _) = sm.tick();
        assert_eq!(alpha, 0.0);
        
        // Press key - becomes visible
        sm.on_key_press();
        let (alpha, _) = sm.tick();
        assert!(alpha > 0.0);
        
        // Wait for visible duration to pass
        sm.visible_duration = Duration::from_millis(50);
        sm.state = BubbleState::Visible { start: Instant::now() - Duration::from_millis(100) };
        let (_, needs_redraw) = sm.tick();
        assert!(needs_redraw);
        
        // Fade out completes
        sm.fade_duration = Duration::from_millis(50);
        sm.state = BubbleState::FadingOut { start: Instant::now() - Duration::from_millis(100) };
        let (alpha, _) = sm.tick();
        assert_eq!(alpha, 0.0);
        assert_eq!(sm.state, BubbleState::Idle);
    }
}
