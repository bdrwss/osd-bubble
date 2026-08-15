use std::sync::{Mutex, Once};
use std::time::{Duration, Instant};
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::HiDpi::*,
};
use tiny_skia::{Color, Paint, Pixmap, Stroke, Transform, FillRule};
use crate::easing::ease_out_cubic;

pub const WM_TRIGGER_RIPPLE: u32 = WM_USER + 10;
pub const WM_RIPPLE_TICK: u32 = WM_USER + 11;

static REGISTER_RIPPLE_CLASS: Once = Once::new();

#[derive(Debug, Clone, Copy)]
pub struct RippleInstance {
    pub x: i32,
    pub y: i32,
    pub color: Color,
    pub birth: Instant,
    pub duration: Duration,
}

lazy_static::lazy_static! {
    pub static ref RIPPLE_STATE: Mutex<Option<RippleInstance>> = Mutex::new(None);
    pub static ref RIPPLE_HWND: Mutex<isize> = Mutex::new(0);
}

pub fn trigger_ripple(x: i32, y: i32, button: rdev::Button) {
    let color = match button {
        rdev::Button::Left => Color::from_rgba8(0, 212, 255, 255),      // 荧光青蓝
        rdev::Button::Right => Color::from_rgba8(255, 152, 0, 255),     // 活力琥珀橙
        rdev::Button::Middle => Color::from_rgba8(16, 185, 129, 255),   // 薄荷翡翠绿
        _ => Color::from_rgba8(0, 212, 255, 255),
    };

    let ripple = RippleInstance {
        x,
        y,
        color,
        birth: Instant::now(),
        duration: Duration::from_millis(260),
    };

    *RIPPLE_STATE.lock().unwrap() = Some(ripple);

    let hwnd_val = *RIPPLE_HWND.lock().unwrap();
    if hwnd_val != 0 {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut _);
            let _ = PostMessageW(Some(hwnd), WM_TRIGGER_RIPPLE, WPARAM(0), LPARAM(0));
        }
    }
}

pub fn show_ripple_overlay(tx: std::sync::mpsc::Sender<isize>) -> Result<()> {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("OsdRippleClass");

        REGISTER_RIPPLE_CLASS.call_once(|| {
            let wc = WNDCLASSW {
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                hInstance: instance.into(),
                lpszClassName: class_name,
                lpfnWndProc: Some(ripple_wndproc),
                ..Default::default()
            };
            RegisterClassW(&wc);
        });

        let ex_style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
        let style = WS_POPUP;

        let hwnd = CreateWindowExW(
            ex_style,
            class_name,
            w!("OsdRipple"),
            style,
            -1000,
            -1000,
            100,
            100,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        *RIPPLE_HWND.lock().unwrap() = hwnd.0 as isize;
        tx.send(hwnd.0 as isize).unwrap();

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

unsafe fn draw_ripple(hwnd: HWND, ripple: &RippleInstance, ripple_size: &str) -> bool {
    let now = Instant::now();
    let elapsed = now.duration_since(ripple.birth);
    if elapsed >= ripple.duration {
        // 动画结束，隐藏窗口
        hide_ripple_window(hwnd);
        return false;
    }

    let p = (elapsed.as_secs_f32() / ripple.duration.as_secs_f32()).clamp(0.0, 1.0);
    let size_max = match ripple_size {
        "small" => 20.0,
        "large" => 40.0,
        _ => 30.0, // medium
    };

    let r = 5.0 + (size_max - 5.0) * ease_out_cubic(p);
    let stroke_w = (2.8 * (1.0 - p)).max(0.6);
    let alpha = (0.92 * (1.0 - p)).clamp(0.0, 1.0);

    let width: u32 = 100;
    let height: u32 = 100;
    let mut pixmap = Pixmap::new(width, height).unwrap();

    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;

    // 1. 外层柔和微光光晕 (Glow Halo)
    let glow_alpha = (alpha * 0.35).clamp(0.0, 1.0);
    let mut gp = Paint::default();
    gp.set_color(Color::from_rgba8(
        (ripple.color.red() * 255.0) as u8,
        (ripple.color.green() * 255.0) as u8,
        (ripple.color.blue() * 255.0) as u8,
        (glow_alpha * 255.0) as u8,
    ));
    gp.anti_alias = true;
    let mut g_stroke = Stroke::default();
    g_stroke.width = stroke_w + 4.0;
    let mut pb_glow = tiny_skia::PathBuilder::new();
    pb_glow.push_circle(cx, cy, r);
    if let Some(path) = pb_glow.finish() {
        pixmap.stroke_path(&path, &gp, &g_stroke, Transform::identity(), None);
    }

    // 2. 主环 (Main Ring)
    let mut mp = Paint::default();
    mp.set_color(Color::from_rgba8(
        (ripple.color.red() * 255.0) as u8,
        (ripple.color.green() * 255.0) as u8,
        (ripple.color.blue() * 255.0) as u8,
        (alpha * 255.0) as u8,
    ));
    mp.anti_alias = true;
    let mut m_stroke = Stroke::default();
    m_stroke.width = stroke_w;
    let mut pb_main = tiny_skia::PathBuilder::new();
    pb_main.push_circle(cx, cy, r);
    if let Some(path) = pb_main.finish() {
        pixmap.stroke_path(&path, &mp, &m_stroke, Transform::identity(), None);
    }

    // 3. 中心微聚光点
    if p < 0.45 {
        let dot_p = p / 0.45;
        let dot_r = (3.5 * (1.0 - dot_p)).max(0.5);
        let mut dp = Paint::default();
        dp.set_color(Color::from_rgba8(255, 255, 255, ((1.0 - dot_p) * 230.0) as u8));
        dp.anti_alias = true;
        let mut pb_dot = tiny_skia::PathBuilder::new();
        pb_dot.push_circle(cx, cy, dot_r);
        if let Some(path) = pb_dot.finish() {
            pixmap.fill_path(&path, &dp, FillRule::Winding, Transform::identity(), None);
        }
    }

    // 更新到 Layered Window
    let hdc_screen = GetDC(None);
    let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD::default(); 1],
    };

    let mut bits = std::ptr::null_mut();
    let hbitmap = CreateDIBSection(Some(hdc_screen), &bmi, DIB_RGB_COLORS, &mut bits, None, 0).unwrap();
    let old_bitmap = SelectObject(hdc_mem, hbitmap.into());

    let src_data = pixmap.data();
    let dst_data = std::slice::from_raw_parts_mut(bits as *mut u8, (width * height * 4) as usize);
    for i in (0..src_data.len()).step_by(4) {
        dst_data[i] = src_data[i + 2];     // Blue
        dst_data[i + 1] = src_data[i + 1]; // Green
        dst_data[i + 2] = src_data[i];     // Red
        dst_data[i + 3] = src_data[i + 3]; // Alpha
    }

    let mut pt_dst = POINT {
        x: ripple.x - (width as i32 / 2),
        y: ripple.y - (height as i32 / 2),
    };
    let mut size = SIZE { cx: width as i32, cy: height as i32 };
    let mut pt_src = POINT { x: 0, y: 0 };
    let mut blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };

    UpdateLayeredWindow(
        hwnd,
        Some(hdc_screen),
        Some(&mut pt_dst),
        Some(&mut size),
        Some(hdc_mem),
        Some(&mut pt_src),
        COLORREF(0),
        Some(&mut blend),
        ULW_ALPHA,
    ).unwrap();

    SelectObject(hdc_mem, old_bitmap);
    let _ = DeleteObject(hbitmap.into());
    let _ = DeleteDC(hdc_mem);
    ReleaseDC(None, hdc_screen);

    true
}

unsafe fn hide_ripple_window(hwnd: HWND) {
    let mut pt_dst = POINT { x: -2000, y: -2000 };
    let mut size = SIZE { cx: 1, cy: 1 };
    let mut pt_src = POINT { x: 0, y: 0 };
    let mut blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 0,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    let _ = UpdateLayeredWindow(
        hwnd,
        None,
        Some(&mut pt_dst),
        Some(&mut size),
        None,
        Some(&mut pt_src),
        COLORREF(0),
        Some(&mut blend),
        ULW_ALPHA,
    );
}

extern "system" fn ripple_wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if message == WM_TRIGGER_RIPPLE || message == WM_RIPPLE_TICK {
            let ripple_opt = *RIPPLE_STATE.lock().unwrap();
            let ripple_size = if let Some(state) = crate::STATE.lock().unwrap().as_ref() {
                state.ripple_size.clone()
            } else {
                "medium".to_string()
            };

            if let Some(ripple) = ripple_opt {
                let still_animating = draw_ripple(window, &ripple, &ripple_size);
                if !still_animating {
                    *RIPPLE_STATE.lock().unwrap() = None;
                }
            }
            return LRESULT(0);
        }

        match message {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
