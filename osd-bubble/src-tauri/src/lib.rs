pub mod overlay;
pub mod hook;
pub mod state_machine;
pub mod renderer;

use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, WPARAM, LPARAM, POINT};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_USER, GetCursorPos};
use state_machine::{StateMachine, CustomStyle};
use tauri::{
    tray::{TrayIconBuilder, MouseButton, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{ShortcutState, GlobalShortcutExt};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW};
use windows::core::PWSTR;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, GetForegroundWindow};

pub const WM_UPDATE_BUBBLE: u32 = WM_USER + 1;
pub const WM_TICK: u32 = WM_USER + 2;

// 全局共享状态机
pub static STATE: Mutex<Option<StateMachine>> = Mutex::new(None);
pub static CURRENT_TEXT: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub static OVERLAY_HWND: Mutex<isize> = Mutex::new(0);

#[tauri::command]
fn update_settings(fade_delay: u64) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.visible_duration = std::time::Duration::from_millis(fade_delay);
    }
}

#[tauri::command]
fn update_position(quadrant: u8) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.quadrant = quadrant;
    }
}

#[tauri::command]
fn update_bubble_style(style: String) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.bubble_style = style;
    }
}

#[tauri::command]
fn update_exclude_apps(apps: Vec<String>) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.exclude_apps = apps;
    }
}

#[tauri::command]
fn update_custom_style(style: CustomStyle) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.custom_style = style;
    }
}


#[tauri::command]
fn toggle_enabled(enabled: bool) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.enabled = enabled;
    }
}

#[tauri::command]
fn update_show_keyboard(show: bool) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.show_keyboard = show;
    }
}

#[tauri::command]
fn update_show_mouse(show: bool) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.show_mouse = show;
    }
}

#[tauri::command]
fn update_show_scroll(show: bool) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.show_scroll = show;
    }
}

#[tauri::command]
fn update_opacity(opacity: f32) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.opacity = opacity.clamp(0.4, 1.0);
    }
}

#[tauri::command]
fn apply_preset(preset: String) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.apply_preset(&preset);
    }
}

#[tauri::command]
fn update_theme(theme: String) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.theme = theme;
    }
}

#[tauri::command]
fn reset_to_defaults() {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.reset_to_defaults();
    }
}

#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn simulate_keys() {
    let keys = vec!["Ctrl".to_string(), "C".to_string()];
    *CURRENT_TEXT.lock().unwrap() = keys;
    
    let mut should_show = false;
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.on_key_press();
        should_show = true;
    }

    if should_show {
        unsafe {
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            let hwnd = HWND(*OVERLAY_HWND.lock().unwrap() as *mut _);
            if !hwnd.0.is_null() {
                let _ = PostMessageW(Some(hwnd), WM_UPDATE_BUBBLE, WPARAM((pt.x + 16) as usize), LPARAM((pt.y + 64) as isize));
            }
        }
    }
}

fn is_foreground_blacklisted(exclude_apps: &[String]) -> bool {
    if exclude_apps.is_empty() {
        return false;
    }
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return false;
        }
        
        let hprocess = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if let Ok(handle) = hprocess {
            let mut buffer = [0u16; 1024];
            let mut size = buffer.len() as u32;
            if QueryFullProcessImageNameW(handle, windows::Win32::System::Threading::PROCESS_NAME_FORMAT(0), PWSTR(buffer.as_mut_ptr()), &mut size).is_ok() {
                let path = String::from_utf16_lossy(&buffer[..size as usize]);
                let filename = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let _ = windows::Win32::Foundation::CloseHandle(handle);
                
                for app in exclude_apps {
                    if filename == app.to_lowercase() {
                        return true;
                    }
                }
            } else {
                let _ = windows::Win32::Foundation::CloseHandle(handle);
            }
        }
    }
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = std::sync::mpsc::channel();
    
    *STATE.lock().unwrap() = Some(StateMachine::new());

    // 启动原生气泡渲染线程
    std::thread::spawn(move || {
        println!("正在创建原生透明窗口...");
        if let Err(e) = overlay::show_overlay(tx) {
            println!("创建窗口失败: {:?}", e);
        }
    });

    let hwnd_isize = rx.recv().expect("未能接收到窗口句柄");
    *OVERLAY_HWND.lock().unwrap() = hwnd_isize;
    let _hwnd = HWND(hwnd_isize as *mut _);

    // 启动一个 60fps 的定时器线程，驱动状态机动画
    std::thread::spawn(move || {
        let timer_hwnd = HWND(hwnd_isize as *mut _);
        loop {
            std::thread::sleep(std::time::Duration::from_millis(16));
            unsafe {
                let _ = PostMessageW(Some(timer_hwnd), WM_TICK, WPARAM(0), LPARAM(0));
            }
        }
    });

    // 启动全局钩子监听线程
    std::thread::spawn(move || {
        println!("正在启动全局钩子监听...");
        let hwnd = HWND(hwnd_isize as *mut _);
        if let Err(error) = rdev::listen(move |event| {
            if let Some(parsed) = hook::parse_event(&event) {
                // 检查是否启用以及事件类型是否允许显示
                let should_process = {
                    if let Some(state) = STATE.lock().unwrap().as_ref() {
                        state.should_show_event(parsed.category)
                    } else {
                        false
                    }
                };

                if should_process {
                    // 更新当前按键文字
                    *CURRENT_TEXT.lock().unwrap() = parsed.keys;
                    
                    // 触发状态机显示
                    let mut should_show = false;
                    if let Some(state) = STATE.lock().unwrap().as_mut() {
                        if !is_foreground_blacklisted(&state.exclude_apps) {
                            state.on_key_press();
                            should_show = true;
                        }
                    }

                    if should_show {
                        // 立即触发一次渲染和移动
                        unsafe {
                            let mut pt = POINT::default();
                            let _ = GetCursorPos(&mut pt);
                            // 将 Y 轴偏移量从 24 增加到 64，以避开输入法候选框
                            let _ = PostMessageW(Some(hwnd), WM_UPDATE_BUBBLE, WPARAM((pt.x + 16) as usize), LPARAM((pt.y + 64) as isize));
                        }
                    }
                }
            }
        }) {
            println!("监听失败: {:?}", error);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hide"])))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
            _ => {}
        })
        .setup(|app| {
            // 根据当前状态生成切换菜单的文字
            let toggle_text = {
                let state = STATE.lock().unwrap();
                if state.as_ref().map_or(true, |s| s.enabled) {
                    "暂停"
                } else {
                    "启用"
                }
            };

            let toggle_i = tauri::menu::MenuItem::with_id(app, "toggle", toggle_text, true, None::<&str>)?;
            let show_i = tauri::menu::MenuItem::with_id(app, "show", "设置", true, None::<&str>)?;
            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&toggle_i, &show_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event({
                    let app_handle = app.handle().clone();
                    move |app, event| match event.id.as_ref() {
                        "quit" => {
                            std::process::exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        "toggle" => {
                            // 切换启用状态
                            let new_enabled = {
                                let mut state = STATE.lock().unwrap();
                                if let Some(ref mut sm) = *state {
                                    sm.enabled = !sm.enabled;
                                    sm.enabled
                                } else {
                                    true
                                }
                            };
                            // 更新托盘菜单文字
                            let new_text = if new_enabled { "暂停" } else { "启用" };
                            if let Ok(new_toggle) = tauri::menu::MenuItem::with_id(app, "toggle", new_text, true, None::<&str>) {
                                let show_item = tauri::menu::MenuItem::with_id(app, "show", "设置", true, None::<&str>).ok();
                                let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>).ok();
                                let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&new_toggle];
                                if let Some(ref s) = show_item { items.push(s); }
                                if let Some(ref q) = quit_item { items.push(q); }
                                if let Ok(new_menu) = tauri::menu::Menu::with_items(app, &items) {
                                    if let Some(tray) = app_handle.tray_by_id("main") {
                                        let _ = tray.set_menu(Some(new_menu));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // 注册全局快捷键
            app.global_shortcut().on_shortcut("Ctrl+Shift+K", move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let mut state = STATE.lock().unwrap();
                    if let Some(ref mut sm) = *state {
                        sm.enabled = !sm.enabled;
                    }
                }
            })?;

            app.global_shortcut().on_shortcut("Ctrl+Shift+,", move |app_handle, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            update_settings,
            update_position,
            update_bubble_style,
            update_exclude_apps,
            update_custom_style,
            simulate_keys,
            toggle_enabled,
            update_show_keyboard,
            update_show_mouse,
            update_show_scroll,
            update_opacity,
            apply_preset,
            update_theme,
            reset_to_defaults,
            get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
