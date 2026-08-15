use crate::platform::PlatformBackend;
use std::error::Error;
use windows::Win32::Foundation::{LPARAM, POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId, PostMessageW,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

pub struct WindowsBackend;

impl WindowsBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformBackend for WindowsBackend {
    fn init_overlay(&self, _app_handle: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
        // Windows 分层透明窗口通过 overlay::start_overlay_window 启动
        Ok(())
    }

    fn update_bubble(&self, x: i32, y: i32) {
        let hwnd_raw = *crate::OVERLAY_HWND.lock().unwrap();
        if hwnd_raw != 0 {
            unsafe {
                let hwnd = windows::Win32::Foundation::HWND(hwnd_raw as *mut _);
                let _ = PostMessageW(
                    Some(hwnd),
                    crate::WM_UPDATE_BUBBLE,
                    WPARAM(x as usize),
                    LPARAM(y as isize),
                );
            }
        }
    }

    fn trigger_ripple(&self, x: i32, y: i32, button: rdev::Button) {
        crate::ripple_overlay::trigger_ripple(x, y, button);
    }

    fn is_foreground_blacklisted(&self, exclude_apps: &[String]) -> bool {
        if exclude_apps.is_empty() {
            return false;
        }
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return false;
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return false;
            }
            let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(h) => h,
                Err(_) => return false,
            };
            let mut buffer = [0u16; 1024];
            let mut size = buffer.len() as u32;
            let success = QueryFullProcessImageNameW(
                process_handle,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR(buffer.as_mut_ptr()),
                &mut size,
            );
            let _ = windows::Win32::Foundation::CloseHandle(process_handle);
            if success.is_ok() {
                let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
                let file_name = std::path::Path::new(&full_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                for app in exclude_apps {
                    if file_name == app.to_lowercase() {
                        return true;
                    }
                }
            }
            false
        }
    }

    fn get_cursor_pos(&self) -> (i32, i32) {
        unsafe {
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            (pt.x, pt.y)
        }
    }
}
