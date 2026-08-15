use crate::platform::PlatformBackend;
use std::error::Error;

pub struct LinuxBackend;

impl LinuxBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformBackend for LinuxBackend {
    fn init_overlay(&self, _app_handle: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
        // Linux 平台透明叠加层初始化
        Ok(())
    }

    fn update_bubble(&self, _x: i32, _y: i32) {
        // 触发 Linux 悬浮窗口重绘
    }

    fn trigger_ripple(&self, _x: i32, _y: i32, _button: rdev::Button) {
        // 触发 Linux 鼠标涟漪
    }

    fn is_foreground_blacklisted(&self, _exclude_apps: &[String]) -> bool {
        // Linux 下通过读取 /proc 或 X11 窗口属性判断
        false
    }

    fn get_cursor_pos(&self) -> (i32, i32) {
        (0, 0)
    }
}
