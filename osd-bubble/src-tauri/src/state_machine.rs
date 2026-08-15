use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::easing::{ease_out_cubic, ease_out_back};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BubbleState {
    Idle,
    FadingIn { start: Instant },
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimFrame {
    pub alpha: f32,
    pub offset_y: f32,
    pub scale: f32,
    pub needs_redraw: bool,
}

pub struct StateMachine {
    pub state: BubbleState,
    pub visible_duration: Duration,
    pub fade_duration: Duration,
    pub fade_in_duration: Duration,
    pub quadrant: u8,
    pub bubble_style: String,
    pub anim_style: String,
    pub exclude_apps: Vec<String>,
    pub custom_style: CustomStyle,
    pub last_strike_time: Instant,
    // V1.0 新增字段
    pub enabled: bool,
    pub show_keyboard: bool,
    pub show_mouse: bool,
    pub show_scroll: bool,
    pub only_shortcuts: bool,
    pub merge_repeats: bool,
    pub opacity: f32,
    pub theme: String,
    pub scale: f32,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: BubbleState::Idle,
            visible_duration: Duration::from_millis(1200),
            fade_duration: Duration::from_millis(280),
            fade_in_duration: Duration::from_millis(120),
            quadrant: 3, // 默认右下
            bubble_style: "default".to_string(),
            anim_style: "bounce".to_string(),
            exclude_apps: Vec::new(),
            custom_style: CustomStyle::new(),
            last_strike_time: Instant::now() - Duration::from_secs(10),
            enabled: true,
            show_keyboard: true,
            show_mouse: true,
            show_scroll: true,
            only_shortcuts: false,
            merge_repeats: true,
            opacity: 0.85,
            theme: "dark".to_string(),
            scale: 1.0,
        }
    }

    /// 根据事件分类判断是否应该显示气泡（默认兼容）
    pub fn should_show_event(&self, category: EventCategory) -> bool {
        self.should_show_event_detailed(category, false)
    }

    /// 根据事件分类及是否包含快捷键组合判断是否应该显示气泡
    pub fn should_show_event_detailed(&self, category: EventCategory, is_shortcut: bool) -> bool {
        if !self.enabled {
            return false;
        }
        match category {
            EventCategory::Keyboard => {
                if !self.show_keyboard {
                    return false;
                }
                if self.only_shortcuts && !is_shortcut {
                    return false;
                }
                true
            }
            EventCategory::Mouse => self.show_mouse,
            EventCategory::Scroll => self.show_scroll,
        }
    }

    pub fn on_key_press(&mut self) {
        let now = Instant::now();
        self.last_strike_time = now;
        // Idle 时先播放入场淡入；其余状态直接重置为可见，
        // 保证连击响应不被入场动画延迟，同时每次击键都重置 last_strike_time 触发微打击动效
        self.state = if matches!(self.state, BubbleState::Idle) {
            BubbleState::FadingIn { start: now }
        } else {
            BubbleState::Visible { start: now }
        };
    }

    /// 综合计算当前帧的多维度动画状态（透明度、Y轴位移、缩放与重绘标记）
    pub fn tick_frame(&mut self) -> AnimFrame {
        if matches!(self.state, BubbleState::Idle) {
            return AnimFrame {
                alpha: 0.0,
                offset_y: 0.0,
                scale: 1.0,
                needs_redraw: false,
            };
        }

        let now = Instant::now();
        let strike_elapsed = now.duration_since(self.last_strike_time);
        
        let (raw_alpha, mut needs_redraw) = match self.state {
            BubbleState::Idle => (0.0, false),
            BubbleState::FadingIn { start } => {
                let elapsed = now.duration_since(start);
                if elapsed >= self.fade_in_duration {
                    self.state = BubbleState::Visible { start: now };
                    (1.0, true)
                } else {
                    let progress = (elapsed.as_secs_f32() / self.fade_in_duration.as_secs_f32()).clamp(0.0, 1.0);
                    let alpha = if self.anim_style == "instant" {
                        1.0
                    } else {
                        ease_out_cubic(progress)
                    };
                    (alpha, true)
                }
            }
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
                    let progress = (elapsed.as_secs_f32() / self.fade_duration.as_secs_f32()).clamp(0.0, 1.0);
                    let alpha = (1.0 - ease_out_cubic(progress)).max(0.0);
                    (alpha, true)
                }
            }
        };

        // 计算 Y 轴位移（向上滑入）
        let mut offset_y = 0.0;
        if self.anim_style == "slide_up" {
            let slide_duration = 0.16; // 160ms
            if strike_elapsed.as_secs_f32() < slide_duration {
                let p = (strike_elapsed.as_secs_f32() / slide_duration).clamp(0.0, 1.0);
                offset_y = (1.0 - ease_out_cubic(p)) * 16.0;
                needs_redraw = true;
            } else if let BubbleState::FadingOut { start } = self.state {
                let p = (now.duration_since(start).as_secs_f32() / self.fade_duration.as_secs_f32()).clamp(0.0, 1.0);
                offset_y = -ease_out_cubic(p) * 10.0;
                needs_redraw = true;
            }
        }

        // 计算弹性缩放（弹性回弹）
        let mut scale = 1.0;
        if self.anim_style == "bounce" {
            let bounce_duration = 0.18; // 180ms
            if strike_elapsed.as_secs_f32() < bounce_duration {
                let p = (strike_elapsed.as_secs_f32() / bounce_duration).clamp(0.0, 1.0);
                scale = 0.85 + 0.15 * ease_out_back(p);
                needs_redraw = true;
            }
        }

        AnimFrame {
            alpha: raw_alpha * self.opacity,
            offset_y,
            scale,
            needs_redraw,
        }
    }

    /// 兼容现有 tick 调用
    pub fn tick(&mut self) -> (f32, bool) {
        let frame = self.tick_frame();
        (frame.alpha, frame.needs_redraw)
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
            // 主题配色预设：只替换配色，不改变气泡形状与时间参数（与前端 stylePresets 保持一致）
            "deep_space" => {
                self.custom_style = CustomStyle {
                    bg_color: "#101418".to_string(),
                    bg_opacity: 0.85,
                    text_color: "#e8eaed".to_string(),
                    border_color: "#2e3a46".to_string(),
                    border_width: 1.0,
                    radius: 12.0,
                    shadow_color: "#000000".to_string(),
                };
            }
            "cream_white" => {
                self.custom_style = CustomStyle {
                    bg_color: "#fdf6ec".to_string(),
                    bg_opacity: 0.95,
                    text_color: "#3d3229".to_string(),
                    border_color: "#e8d5b7".to_string(),
                    border_width: 1.0,
                    radius: 12.0,
                    shadow_color: "#8a7a63".to_string(),
                };
            }
            "neon_blue" => {
                self.custom_style = CustomStyle {
                    bg_color: "#0a1a2f".to_string(),
                    bg_opacity: 0.9,
                    text_color: "#7df9ff".to_string(),
                    border_color: "#00d4ff".to_string(),
                    border_width: 1.5,
                    radius: 10.0,
                    shadow_color: "#001f33".to_string(),
                };
            }
            _ => {}
        }
    }

    /// 重置所有字段为默认值
    pub fn reset_to_defaults(&mut self) {
        *self = Self::new();
    }

    /// 从持久化的设置 JSON 恢复运行时字段。
    /// 逐字段防御式解析：缺失或非法的字段保持当前值不变。
    /// 注意：不重置 state（动画状态），也不处理前端专有字段（theme/autoStart 等）。
    pub fn apply_persisted_settings(&mut self, value: &serde_json::Value) {
        let Some(obj) = value.as_object() else { return };

        if let Some(ms) = obj.get("fadeDelay").and_then(|v| v.as_u64()) {
            self.visible_duration = Duration::from_millis(ms);
        }
        if let Some(ms) = obj.get("fadeInDuration").and_then(|v| v.as_u64()) {
            self.fade_in_duration = Duration::from_millis(ms);
        }
        if let Some(pct) = obj.get("opacity").and_then(|v| v.as_f64()) {
            self.opacity = ((pct / 100.0).clamp(0.4, 1.0)) as f32;
        }
        if let Some(q) = obj.get("quadrant").and_then(|v| v.as_str()).and_then(|s| s.parse::<u8>().ok()) {
            if q <= 3 {
                self.quadrant = q;
            }
        }
        if let Some(style) = obj.get("bubbleStyle").and_then(|v| v.as_str()) {
            self.bubble_style = style.to_string();
        }
        if let Some(style) = obj.get("animStyle").and_then(|v| v.as_str()) {
            self.anim_style = style.to_string();
        }
        if let Some(enabled) = obj.get("enabled").and_then(|v| v.as_bool()) {
            self.enabled = enabled;
        }
        if let Some(show) = obj.get("showKeyboard").and_then(|v| v.as_bool()) {
            self.show_keyboard = show;
        }
        if let Some(show) = obj.get("showMouse").and_then(|v| v.as_bool()) {
            self.show_mouse = show;
        }
        if let Some(show) = obj.get("showScroll").and_then(|v| v.as_bool()) {
            self.show_scroll = show;
        }
        if let Some(only) = obj.get("onlyShortcuts").and_then(|v| v.as_bool()) {
            self.only_shortcuts = only;
        }
        if let Some(merge) = obj.get("mergeRepeats").and_then(|v| v.as_bool()) {
            self.merge_repeats = merge;
        }
        if let Some(apps) = obj.get("excludeApps").and_then(|v| v.as_array()) {
            let list: Vec<String> = apps.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
            self.exclude_apps = list;
        }
        if let Some(cs) = obj.get("customStyle") {
            if let Ok(style) = serde_json::from_value::<CustomStyle>(cs.clone()) {
                self.custom_style = style;
            }
        }
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
        assert_eq!(sm.fade_in_duration, Duration::from_millis(120));
        assert!(sm.enabled);
        assert!(sm.show_keyboard);
        assert!(sm.show_mouse);
        assert!(sm.show_scroll);
    }

    #[test]
    fn test_on_key_press_from_idle_transitions_to_fading_in() {
        let mut sm = StateMachine::new();
        sm.state = BubbleState::Idle;
        
        sm.on_key_press();
        
        assert!(matches!(sm.state, BubbleState::FadingIn { .. }));
    }

    #[test]
    fn test_on_key_press_during_fading_in_jumps_to_visible() {
        let mut sm = StateMachine::new();
        sm.state = BubbleState::FadingIn { start: Instant::now() };
        
        sm.on_key_press();
        
        // 连击时跳过入场动画，直接完全可见，不延迟响应
        assert!(matches!(sm.state, BubbleState::Visible { .. }));
    }

    #[test]
    fn test_on_key_press_during_fading_out_resets_to_visible() {
        let mut sm = StateMachine::new();
        sm.state = BubbleState::FadingOut { start: Instant::now() };
        
        sm.on_key_press();
        
        assert!(matches!(sm.state, BubbleState::Visible { .. }));
    }

    #[test]
    fn test_tick_fading_in_alpha_increases() {
        let mut sm = StateMachine::new();
        sm.fade_in_duration = Duration::from_millis(1000);
        sm.opacity = 1.0;
        sm.state = BubbleState::FadingIn { start: Instant::now() - Duration::from_millis(200) };
        
        let (alpha1, redraw1) = sm.tick();
        let (alpha2, redraw2) = sm.tick();
        
        assert!(alpha1 > 0.0 && alpha1 < 1.0);
        assert!(alpha2 >= alpha1, "淡入过程中 alpha 应递增");
        assert!(redraw1 && redraw2);
        assert!(matches!(sm.state, BubbleState::FadingIn { .. }));
    }

    #[test]
    fn test_tick_fading_in_completes_to_visible() {
        let mut sm = StateMachine::new();
        sm.fade_in_duration = Duration::from_millis(100);
        sm.state = BubbleState::FadingIn { start: Instant::now() - Duration::from_millis(150) };
        
        let (alpha, needs_redraw) = sm.tick();
        
        assert_eq!(alpha, 0.85); // 默认 opacity
        assert!(needs_redraw);
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
        // easeOutCubic 淡出：前 1/3 时长应完成超过 50% 的 alpha 下降（先快后慢）
        sm.state = BubbleState::FadingOut { start: Instant::now() - Duration::from_millis(333) };
        let (alpha_early, _) = sm.tick();
        assert!(alpha_early < 0.5, "淡出应前快后慢，前 1/3 时长 alpha 应已降至 0.5 以下");
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
    fn test_apply_theme_preset_only_changes_colors() {
        // 主题配色预设不应改变气泡形状与时间参数
        for preset in ["deep_space", "cream_white", "neon_blue"] {
            let mut sm = StateMachine::new();
            let duration_before = sm.visible_duration;
            let style_before = sm.bubble_style.clone();
            let scale_before = sm.scale;

            sm.apply_preset(preset);

            assert_eq!(sm.visible_duration, duration_before, "{preset} 不应改显示时长");
            assert_eq!(sm.bubble_style, style_before, "{preset} 不应改气泡形状");
            assert_eq!(sm.scale, scale_before, "{preset} 不应改缩放");
            assert_ne!(sm.custom_style.bg_color, "#000000", "{preset} 应替换配色");
        }
    }

    #[test]
    fn test_apply_theme_preset_values() {
        let mut sm = StateMachine::new();
        sm.apply_preset("neon_blue");
        assert_eq!(sm.custom_style.bg_color, "#0a1a2f");
        assert_eq!(sm.custom_style.text_color, "#7df9ff");
        assert!((sm.custom_style.bg_opacity - 0.9).abs() < 1e-6);
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
        
        // Press key - becomes fading in
        sm.on_key_press();
        assert!(matches!(sm.state, BubbleState::FadingIn { .. }));
        
        // Fade in completes
        sm.fade_in_duration = Duration::from_millis(50);
        sm.state = BubbleState::FadingIn { start: Instant::now() - Duration::from_millis(100) };
        let (alpha, needs_redraw) = sm.tick();
        assert!(alpha > 0.0);
        assert!(needs_redraw);
        assert!(matches!(sm.state, BubbleState::Visible { .. }));
        
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

    #[test]
    fn test_apply_persisted_settings_full() {
        let mut sm = StateMachine::new();
        let json = serde_json::json!({
            "fadeDelay": 2000,
            "fadeInDuration": 200,
            "opacity": 60,
            "quadrant": "1",
            "bubbleStyle": "cartoon",
            "enabled": false,
            "showKeyboard": false,
            "showMouse": true,
            "showScroll": true,
            "excludeApps": ["csgo.exe"],
            "customStyle": {
                "bg_color": "#112233",
                "bg_opacity": 0.5,
                "text_color": "#ffffff",
                "border_color": "#000000",
                "border_width": 2.0,
                "radius": 4.0,
                "shadow_color": "#000000"
            },
            "theme": "light",
            "autoStart": true
        });

        sm.apply_persisted_settings(&json);

        assert_eq!(sm.visible_duration, Duration::from_millis(2000));
        assert_eq!(sm.fade_in_duration, Duration::from_millis(200));
        assert!((sm.opacity - 0.6).abs() < 1e-6);
        assert_eq!(sm.quadrant, 1);
        assert_eq!(sm.bubble_style, "cartoon");
        assert!(!sm.enabled);
        assert!(!sm.show_keyboard);
        assert!(sm.show_mouse);
        assert!(sm.show_scroll);
        assert_eq!(sm.exclude_apps, vec!["csgo.exe".to_string()]);
        assert_eq!(sm.custom_style.bg_color, "#112233");
        assert!((sm.custom_style.border_width - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_persisted_settings_empty_object_keeps_defaults() {
        let mut sm = StateMachine::new();
        let before = sm.visible_duration;
        sm.apply_persisted_settings(&serde_json::json!({}));
        assert_eq!(sm.visible_duration, before);
        assert!(sm.enabled);
    }

    #[test]
    fn test_apply_persisted_settings_invalid_json_ignored() {
        let mut sm = StateMachine::new();
        sm.apply_persisted_settings(&serde_json::json!("not an object"));
        sm.apply_persisted_settings(&serde_json::json!(42));
        assert_eq!(sm.visible_duration, Duration::from_millis(1200));
        assert_eq!(sm.quadrant, 3);
    }

    #[test]
    fn test_apply_persisted_settings_invalid_fields_skipped() {
        let mut sm = StateMachine::new();
        let json = serde_json::json!({
            "fadeDelay": "abc",
            "fadeInDuration": "abc",
            "quadrant": "9",
            "customStyle": { "bg_color": "#000000" }
        });
        sm.apply_persisted_settings(&json);
        assert_eq!(sm.visible_duration, Duration::from_millis(1200));
        assert_eq!(sm.fade_in_duration, Duration::from_millis(120));
        assert_eq!(sm.quadrant, 3);
        // customStyle 缺少字段，反序列化失败，保持默认
        assert_eq!(sm.custom_style.bg_color, "#000000");
        assert!((sm.custom_style.bg_opacity - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_apply_persisted_settings_opacity_clamped() {
        let mut sm = StateMachine::new();
        sm.apply_persisted_settings(&serde_json::json!({ "opacity": 10 }));
        assert!((sm.opacity - 0.4).abs() < 1e-6);
        sm.apply_persisted_settings(&serde_json::json!({ "opacity": 500 }));
        assert!((sm.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_should_show_event_detailed_shortcuts() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.only_shortcuts, false);
        
        // 默认模式：无论是否为快捷键，键盘事件都显示
        assert!(sm.should_show_event_detailed(EventCategory::Keyboard, false));
        assert!(sm.should_show_event_detailed(EventCategory::Keyboard, true));

        // 开启仅显示快捷键模式
        sm.only_shortcuts = true;
        assert!(!sm.should_show_event_detailed(EventCategory::Keyboard, false)); // 普通打字被过滤
        assert!(sm.should_show_event_detailed(EventCategory::Keyboard, true));  // 快捷键正常显示

        // 鼠标和滚轮不受影响
        assert!(sm.should_show_event_detailed(EventCategory::Mouse, false));
        assert!(sm.should_show_event_detailed(EventCategory::Scroll, false));

        // 禁用键盘后，快捷键也不显示
        sm.show_keyboard = false;
        assert!(!sm.should_show_event_detailed(EventCategory::Keyboard, true));
    }

    #[test]
    fn test_apply_persisted_settings_only_shortcuts() {
        let mut sm = StateMachine::new();
        assert!(!sm.only_shortcuts);
        sm.apply_persisted_settings(&serde_json::json!({ "onlyShortcuts": true }));
        assert!(sm.only_shortcuts);
        sm.apply_persisted_settings(&serde_json::json!({ "onlyShortcuts": false }));
        assert!(!sm.only_shortcuts);
    }

    #[test]
    fn test_apply_persisted_settings_merge_repeats() {
        let mut sm = StateMachine::new();
        assert!(sm.merge_repeats);
        sm.apply_persisted_settings(&serde_json::json!({ "mergeRepeats": false }));
        assert!(!sm.merge_repeats);
        sm.apply_persisted_settings(&serde_json::json!({ "mergeRepeats": true }));
        assert!(sm.merge_repeats);
    }

    #[test]
    fn test_apply_persisted_settings_anim_style() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.anim_style, "bounce");
        sm.apply_persisted_settings(&serde_json::json!({ "animStyle": "fade" }));
        assert_eq!(sm.anim_style, "fade");
        sm.apply_persisted_settings(&serde_json::json!({ "animStyle": "instant" }));
        assert_eq!(sm.anim_style, "instant");
    }

    #[test]
    fn test_tick_anim_style_curves() {
        let mut sm = StateMachine::new();
        sm.fade_in_duration = Duration::from_millis(100);
        sm.opacity = 1.0;

        // 测试 bounce 曲线
        sm.anim_style = "bounce".to_string();
        sm.state = BubbleState::FadingIn { start: Instant::now() - Duration::from_millis(50) };
        let (alpha_bounce, redraw) = sm.tick();
        assert!(redraw);
        assert!(alpha_bounce > 0.0 && alpha_bounce <= 1.0);

        // 测试 instant 曲线（直接返回 1.0）
        sm.anim_style = "instant".to_string();
        sm.state = BubbleState::FadingIn { start: Instant::now() - Duration::from_millis(10) };
        let (alpha_instant, redraw_inst) = sm.tick();
        assert!(redraw_inst);
        assert_eq!(alpha_instant, 1.0);
    }

    #[test]
    fn test_tick_frame_slide_up_offset() {
        let mut sm = StateMachine::new();
        sm.state = BubbleState::Visible { start: Instant::now() };
        sm.anim_style = "slide_up".to_string();
        sm.last_strike_time = Instant::now() - Duration::from_millis(40);
        let frame = sm.tick_frame();
        assert!(frame.offset_y > 0.0);
    }

    #[test]
    fn test_tick_frame_bounce_scale() {
        let mut sm = StateMachine::new();
        sm.state = BubbleState::Visible { start: Instant::now() };
        sm.anim_style = "bounce".to_string();
        sm.last_strike_time = Instant::now() - Duration::from_millis(60);
        let frame = sm.tick_frame();
        // 处于过冲回弹区间，scale 明显大于 0.85
        assert!(frame.scale > 0.90);
    }
}
