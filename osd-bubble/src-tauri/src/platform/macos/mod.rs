use crate::platform::PlatformBackend;
use std::error::Error;

pub struct MacosBackend;

impl MacosBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformBackend for MacosBackend {
    fn init_overlay(&self, _app_handle: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
        // macOS 平台透明悬浮层初始化
        Ok(())
    }

    fn update_bubble(&self, _x: i32, _y: i32) {
        // 触发 macOS 悬浮窗口重绘
    }

    fn trigger_ripple(&self, _x: i32, _y: i32, _button: rdev::Button) {
        // 触发 macOS 鼠标涟漪
    }

    fn is_foreground_blacklisted(&self, exclude_apps: &[String]) -> bool {
        if exclude_apps.is_empty() {
            return false;
        }
        // macOS 下通过 NSWorkspace 获取当前活跃 App 名称
        false
    }

    fn get_cursor_pos(&self) -> (i32, i32) {
        (0, 0)
    }

    fn check_permissions(&self) -> bool {
        // macOS 辅助功能权限检测
        true
    }
}
