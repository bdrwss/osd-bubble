use std::error::Error;

pub trait PlatformBackend: Send + Sync {
    /// 初始化悬浮叠加层窗口与环境
    fn init_overlay(&self, app_handle: &tauri::AppHandle) -> Result<(), Box<dyn Error>>;
    /// 触发按键气泡更新渲染
    fn update_bubble(&self, x: i32, y: i32);
    /// 触发鼠标点击光环扩散
    fn trigger_ripple(&self, x: i32, y: i32, button: rdev::Button);
    /// 检查当前前台应用是否在黑名单中
    fn is_foreground_blacklisted(&self, exclude_apps: &[String]) -> bool;
    /// 获取当前鼠标光标物理坐标 (x, y)
    fn get_cursor_pos(&self) -> (i32, i32);
    /// 检查并请求辅助功能/输入监听权限 (macOS 专属，其他平台默认返回 true)
    fn check_permissions(&self) -> bool {
        true
    }
}

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::WindowsBackend as CurrentPlatform;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::MacosBackend as CurrentPlatform;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::LinuxBackend as CurrentPlatform;

pub fn create_platform() -> Box<dyn PlatformBackend> {
    Box::new(CurrentPlatform::new())
}
