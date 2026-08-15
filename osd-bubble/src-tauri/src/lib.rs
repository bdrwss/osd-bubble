pub mod overlay;
pub mod hook;
pub mod state_machine;
pub mod renderer;
pub mod easing;
pub mod ripple_overlay;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use windows::Win32::Foundation::{HWND, WPARAM, LPARAM, POINT};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_USER, GetCursorPos};
use state_machine::{StateMachine, CustomStyle};
use tauri::{
    tray::{TrayIconBuilder, MouseButton, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{ShortcutState, GlobalShortcutExt};
use tauri_plugin_store::StoreExt;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW};
use windows::core::PWSTR;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, GetForegroundWindow};

pub const WM_UPDATE_BUBBLE: u32 = WM_USER + 1;
pub const WM_TICK: u32 = WM_USER + 2;

// 全局共享状态机
pub static STATE: Mutex<Option<StateMachine>> = Mutex::new(None);
pub static CURRENT_TEXT: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub static OVERLAY_HWND: Mutex<isize> = Mutex::new(0);
// 连击乘数（"×N"）首次出现的时间戳，用于渲染端入场动画；无乘数时为 None
pub static MULTIPLIER_BIRTH: Mutex<Option<Instant>> = Mutex::new(None);

/// 更新当前按键文字，并同步维护乘数出生时间戳：
/// 乘数已存在则保留首次出现时间（连击递增不重播动画），消失则清空
fn set_current_text(keys: Vec<String>) {
    let has_multiplier = keys.iter().any(|k| k.starts_with('×'));
    let mut birth = MULTIPLIER_BIRTH.lock().unwrap();
    if has_multiplier {
        if birth.is_none() {
            *birth = Some(Instant::now());
        }
    } else {
        *birth = None;
    }
    let mult_birth = *birth;
    *CURRENT_TEXT.lock().unwrap() = keys.clone();
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.push_history(keys, mult_birth);
    }
}

// TASK-001 缺陷A诊断：钩子启动期事件统计（用于定位启动后键盘事件暂时无效的问题）
static HOOK_BOOT_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
static HOOK_EVENT_COUNT: AtomicUsize = AtomicUsize::new(0);
static HOOK_FIRST_KEY_LOGGED: AtomicBool = AtomicBool::new(false);
static HOOK_FIRST_MOUSE_LOGGED: AtomicBool = AtomicBool::new(false);

fn log_hook_event(event_type: &rdev::EventType) {
    let n = HOOK_EVENT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    let ms = HOOK_BOOT_TIME.get().map(|t| t.elapsed().as_millis()).unwrap_or(0);
    let kind = match event_type {
        rdev::EventType::KeyPress(_) => "KeyPress",
        rdev::EventType::KeyRelease(_) => "KeyRelease",
        rdev::EventType::ButtonPress(_) => "ButtonPress",
        rdev::EventType::ButtonRelease(_) => "ButtonRelease",
        rdev::EventType::Wheel { .. } => "Wheel",
        rdev::EventType::MouseMove { .. } => "MouseMove",
    };
    if n <= 5 {
        println!("[hook-diag] event #{} at +{}ms: {}", n, ms, kind);
    }
    let is_key = matches!(event_type, rdev::EventType::KeyPress(_) | rdev::EventType::KeyRelease(_));
    if is_key && !HOOK_FIRST_KEY_LOGGED.swap(true, Ordering::Relaxed) {
        println!("[hook-diag] first keyboard event at +{}ms (event #{})", ms, n);
    }
    if !is_key && !HOOK_FIRST_MOUSE_LOGGED.swap(true, Ordering::Relaxed) {
        println!("[hook-diag] first mouse event at +{}ms (event #{})", ms, n);
    }
}

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
fn update_position_mode(mode: String) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.position_mode = mode;
    }
}

#[tauri::command]
fn update_screen_anchor(anchor: String) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.screen_anchor = anchor;
    }
}

#[tauri::command]
fn update_anchor_margins(margin_x: i32, margin_y: i32) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.anchor_margin_x = margin_x.clamp(0, 300);
        state.anchor_margin_y = margin_y.clamp(0, 300);
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
fn update_only_shortcuts(only: bool) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.only_shortcuts = only;
    }
}

#[tauri::command]
fn update_merge_repeats(merge: bool) {
    hook::set_merge_repeats(merge);
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.merge_repeats = merge;
    }
}

#[tauri::command]
fn update_anim_style(style: String) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.anim_style = style;
    }
}

#[tauri::command]
fn update_enable_history(enable: bool) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.enable_history = enable;
    }
}

#[tauri::command]
fn update_max_history(max: usize) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.max_history = max.clamp(1, 10);
    }
}

#[tauri::command]
fn update_enable_mouse_ripple(enable: bool) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.enable_mouse_ripple = enable;
    }
}

#[tauri::command]
fn update_ripple_size(size: String) {
    if let Some(state) = STATE.lock().unwrap().as_mut() {
        state.ripple_size = size;
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
    set_current_text(keys);
    
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
    let (tx_ripple, rx_ripple) = std::sync::mpsc::channel();
    
    *STATE.lock().unwrap() = Some(StateMachine::new());

    // 启动原生气泡渲染线程
    std::thread::spawn(move || {
        println!("正在创建原生透明窗口...");
        if let Err(e) = overlay::show_overlay(tx) {
            println!("创建窗口失败: {:?}", e);
        }
    });

    // 启动独立鼠标点击涟漪光环窗口线程
    std::thread::spawn(move || {
        if let Err(e) = ripple_overlay::show_ripple_overlay(tx_ripple) {
            println!("创建鼠标涟漪窗口失败: {:?}", e);
        }
    });

    let hwnd_isize = rx.recv().expect("未能接收到窗口句柄");
    *OVERLAY_HWND.lock().unwrap() = hwnd_isize;

    let ripple_hwnd_isize = rx_ripple.recv().unwrap_or(0);

    // 启动一个 60fps 的定时器线程，驱动状态机动画与鼠标涟漪动画
    std::thread::spawn(move || {
        let timer_hwnd = HWND(hwnd_isize as *mut _);
        let ripple_hwnd = if ripple_hwnd_isize != 0 { Some(HWND(ripple_hwnd_isize as *mut _)) } else { None };
        loop {
            std::thread::sleep(std::time::Duration::from_millis(16));
            unsafe {
                let _ = PostMessageW(Some(timer_hwnd), WM_TICK, WPARAM(0), LPARAM(0));
                if let Some(rhw) = ripple_hwnd {
                    if ripple_overlay::RIPPLE_STATE.lock().unwrap().is_some() {
                        let _ = PostMessageW(Some(rhw), ripple_overlay::WM_RIPPLE_TICK, WPARAM(0), LPARAM(0));
                    }
                }
            }
        }
    });

    // 全局钩子监听线程已移至 setup 中启动（store 恢复设置之后），
    // 避免启动窗口期内以默认配置处理事件（TASK-001 缺陷B）

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
            // 从持久化 store 恢复设置，使后端不依赖前端窗口加载即可持有正确配置
            // 必须在构建托盘菜单之前执行，保证"暂停/启用"菜单文字与真实状态一致
            match app.store("settings.json") {
                Ok(store) => {
                    if let Some(json) = store.get("osdBubbleSettings") {
                        if let Some(state) = STATE.lock().unwrap().as_mut() {
                            state.apply_persisted_settings(&json);
                            hook::set_merge_repeats(state.merge_repeats);
                        }
                    }
                }
                Err(e) => {
                    println!("加载设置 store 失败，使用默认配置: {:?}", e);
                }
            }

            // 启动全局钩子监听线程
            // 必须在设置恢复之后：保证捕获到的事件按持久化配置处理，
            // 消除启动窗口期内以默认配置显示气泡的问题（TASK-001 缺陷B）
            std::thread::spawn(move || {
                println!("正在启动全局钩子监听...");
                let hwnd = HWND(*OVERLAY_HWND.lock().unwrap() as *mut _);
                HOOK_BOOT_TIME.get_or_init(Instant::now);
                if let Err(error) = rdev::listen(move |event| {
                    log_hook_event(&event.event_type);

                    // 鼠标点击光环与涟漪触发
                    if let rdev::EventType::ButtonPress(button) = event.event_type {
                        let (enable_ripple, is_blacklisted) = {
                            if let Some(state) = STATE.lock().unwrap().as_ref() {
                                (state.enable_mouse_ripple && state.enabled, is_foreground_blacklisted(&state.exclude_apps))
                            } else {
                                (false, false)
                            }
                        };
                        if enable_ripple && !is_blacklisted {
                            unsafe {
                                let mut pt = POINT::default();
                                let _ = GetCursorPos(&mut pt);
                                ripple_overlay::trigger_ripple(pt.x, pt.y, button);
                            }
                        }
                    }

                    if let Some(parsed) = hook::parse_event(&event) {
                        // 检查是否启用以及事件类型是否允许显示
                        let should_process = {
                            if let Some(state) = STATE.lock().unwrap().as_ref() {
                                state.should_show_event_detailed(parsed.category, parsed.is_shortcut)
                            } else {
                                false
                            }
                        };

                        if should_process {
                            // 更新当前按键文字（同步维护乘数出生时间戳）
                            set_current_text(parsed.keys);

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
            let icon = app.default_window_icon().cloned().expect("failed to get default window icon");

            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon.clone())
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

            // 显式确保主窗口应用最新图标
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(icon);
            }

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
            update_position_mode,
            update_screen_anchor,
            update_anchor_margins,
            update_bubble_style,
            update_exclude_apps,
            update_custom_style,
            simulate_keys,
            toggle_enabled,
            update_show_keyboard,
            update_show_mouse,
            update_show_scroll,
            update_only_shortcuts,
            update_merge_repeats,
            update_anim_style,
            update_enable_history,
            update_max_history,
            update_enable_mouse_ripple,
            update_ripple_size,
            update_opacity,
            apply_preset,
            update_theme,
            reset_to_defaults,
            get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
