use std::sync::Once;

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::HiDpi::*,
};
use std::sync::Mutex;
use crate::renderer::BubbleRenderer;

static REGISTER_CLASS: Once = Once::new();

lazy_static::lazy_static! {
    static ref RENDERER: Mutex<BubbleRenderer> = Mutex::new(BubbleRenderer::new());
}

pub fn show_overlay(tx: std::sync::mpsc::Sender<isize>) -> Result<()> {
    unsafe {
        // 开启 V2 版 Per-Monitor DPI 感知，保证高分屏不模糊
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let instance = GetModuleHandleW(None)?;
        let class_name = w!("OsdBubbleClass");

        REGISTER_CLASS.call_once(|| {
            let wc = WNDCLASSW {
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                hInstance: instance.into(),
                lpszClassName: class_name,
                lpfnWndProc: Some(wndproc),
                ..Default::default()
            };
            RegisterClassW(&wc);
        });

        // 创建 WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW
        let ex_style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
        let style = WS_POPUP;

        let hwnd = CreateWindowExW(
            ex_style,
            class_name,
            w!("OsdBubble"),
            style,
            100, // x
            100, // y
            300, // width
            100, // height
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        // 通知主线程 HWND 已经准备好
        tx.send(hwnd.0 as isize).unwrap();

        // 初始绘制隐藏在屏幕外或默认位置
        draw_bubble(
            hwnd,
            POINT { x: -1000, y: -1000 },
            crate::state_machine::AnimFrame { alpha: 0.0, offset_y: 0.0, scale: 1.0, needs_redraw: false },
            3,
            "default",
            &crate::state_machine::CustomStyle::new()
        );

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

unsafe fn draw_bubble(hwnd: HWND, pt: POINT, frame: crate::state_machine::AnimFrame, quadrant: u8, style: &str, custom: &crate::state_machine::CustomStyle) {
    let mut history_items = Vec::new();
    if let Some(state) = crate::STATE.lock().unwrap().as_mut() {
        let (items, _) = state.get_active_history();
        history_items = items;
    }
    
    if history_items.is_empty() {
        let keys = crate::CURRENT_TEXT.lock().unwrap().clone();
        if keys.is_empty() { return; }
        let multiplier_birth = *crate::MULTIPLIER_BIRTH.lock().unwrap();
        history_items.push(crate::state_machine::RenderHistoryItem {
            keys,
            multiplier_birth,
            alpha: 1.0,
        });
    }

    let pixmap = RENDERER.lock().unwrap().draw_history(&history_items, style, custom, frame.scale);
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    let mut bx = match quadrant {
        0 | 2 => pt.x - width - 16,
        _ => pt.x + 16,
    };
    let mut by = match quadrant {
        0 | 1 => pt.y - height - 24,
        _ => pt.y + 24,
    };

    // 应用动画 Y 轴位移（向上滑入等）
    by = (by as f32 + frame.offset_y) as i32;

    let hmonitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(hmonitor, &mut mi as *mut _ as *mut _);
    
    let work_area = mi.rcWork;
    if bx + width > work_area.right {
        bx = pt.x - width - 16;
    }
    if bx < work_area.left {
        bx = pt.x + 16;
    }
    if by + height > work_area.bottom {
        by = pt.y - height - 24;
    }
    if by < work_area.top {
        by = pt.y + 24;
    }

    // 将 pixmap 数据更新到 Layered Window
    let hdc_screen = GetDC(None);
    let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // top-down
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

    // 拷贝像素并进行 RGBA 到 BGRA 的转换
    let src_data = pixmap.data();
    let dst_data = unsafe { std::slice::from_raw_parts_mut(bits as *mut u8, (width * height * 4) as usize) };
    
    for i in (0..src_data.len()).step_by(4) {
        dst_data[i] = src_data[i + 2];     // Blue
        dst_data[i + 1] = src_data[i + 1]; // Green
        dst_data[i + 2] = src_data[i];     // Red
        dst_data[i + 3] = src_data[i + 3]; // Alpha
    }

    let mut pt_dst = POINT { x: bx, y: by };
    let mut size = SIZE { cx: width, cy: height };
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
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if message == crate::WM_TICK || message == crate::WM_UPDATE_BUBBLE {
            let mut frame = crate::state_machine::AnimFrame {
                alpha: 0.0,
                offset_y: 0.0,
                scale: 1.0,
                needs_redraw: false,
            };
            let mut quadrant = 3;
            let mut bubble_style = "default".to_string();
            let mut custom_style = crate::state_machine::CustomStyle::new();
            let mut history_animating = false;

            if let Some(state) = crate::STATE.lock().unwrap().as_mut() {
                frame = state.tick_frame();
                quadrant = state.quadrant;
                bubble_style = state.bubble_style.clone();
                custom_style = state.custom_style.clone();
                let (_, anim) = state.get_active_history();
                history_animating = anim;
            }

            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            
            static mut LAST_CURSOR: POINT = POINT { x: 0, y: 0 };
            
            if frame.alpha > 0.0 || history_animating {
                // 持续置顶，防止被右键菜单或其他置顶窗口覆盖
                let _ = SetWindowPos(window, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                
                if pt.x != LAST_CURSOR.x || pt.y != LAST_CURSOR.y {
                    frame.needs_redraw = true;
                    LAST_CURSOR = pt;
                }

                // 连击乘数入场动画进行中时持续重绘
                if let Some(birth) = *crate::MULTIPLIER_BIRTH.lock().unwrap() {
                    if birth.elapsed() < std::time::Duration::from_millis(150) {
                        frame.needs_redraw = true;
                    }
                }

                if history_animating {
                    frame.needs_redraw = true;
                }
            }

            if message == crate::WM_UPDATE_BUBBLE || frame.needs_redraw {
                draw_bubble(window, pt, frame, quadrant, &bubble_style, &custom_style);
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
