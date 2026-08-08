pub mod text;
pub mod layout;

use tiny_skia::{Pixmap, Color, Paint, Stroke, Transform, PathBuilder, FillRule, Path, LinearGradient, GradientStop};
use text::FontRenderer;
use layout::BubbleLayout;
use crate::easing::ease_out_cubic;

pub struct BubbleRenderer {
    font_renderer: FontRenderer,
}

fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let mut pb = PathBuilder::new();
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = 0.552284749831 * r;
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish().unwrap()
}

fn parse_hex_color(hex: &str, opacity: f32) -> Color {
    let hex = hex.trim_start_matches('#');
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    if hex.len() >= 6 {
        r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    }
    let a = (opacity * 255.0).clamp(0.0, 255.0) as u8;
    Color::from_rgba8(r, g, b, a)
}

/// 按系数调整颜色明度：amount > 0 提亮（向白混合），amount < 0 压暗（向黑混合）
fn adjust_lightness(c: Color, amount: f32) -> Color {
    let (r, g, b) = if amount >= 0.0 {
        (
            c.red() + (1.0 - c.red()) * amount,
            c.green() + (1.0 - c.green()) * amount,
            c.blue() + (1.0 - c.blue()) * amount,
        )
    } else {
        let f = 1.0 + amount;
        (c.red() * f, c.green() * f, c.blue() * f)
    };
    Color::from_rgba8(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (c.alpha() * 255.0) as u8,
    )
}

/// 软阴影参数：外扩像素与对应的 alpha（0–255），逐层递减模拟柔和投影
const SHADOW_LAYERS: [(f32, u8); 4] = [(2.0, 40), (4.0, 25), (6.0, 15), (8.0, 8)];
/// default 风格软阴影需要在画布四周预留的空间
const SHADOW_PADDING: f32 = 12.0;
/// 连击乘数入场动画时长与起始缩放（150ms 淡入 + 0.8→1.0 微缩放）
const MULTIPLIER_ANIM_MS: f32 = 150.0;
const MULTIPLIER_SCALE_FROM: f32 = 0.8;

/// 计算连击乘数入场动画进度（0.0–1.0）：无出生时间戳表示动画已完成
pub fn multiplier_anim_progress(birth: Option<std::time::Instant>, now: std::time::Instant) -> f32 {
    let Some(b) = birth else { return 1.0 };
    let elapsed = now.duration_since(b).as_secs_f32() * 1000.0;
    if elapsed >= MULTIPLIER_ANIM_MS {
        return 1.0;
    }
    ease_out_cubic(elapsed / MULTIPLIER_ANIM_MS)
}

impl Default for BubbleRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl BubbleRenderer {
    pub fn new() -> Self {
        Self {
            font_renderer: FontRenderer::new(),
        }
    }

    pub fn draw(&self, keys: &[String], style: &str, custom: &crate::state_machine::CustomStyle, multiplier_birth: Option<std::time::Instant>) -> Pixmap {
        let layout = BubbleLayout::build(keys);
        
        // default 风格带软阴影，画布四周预留 SHADOW_PADDING，其余风格不预留
        let pad = if style == "default" { SHADOW_PADDING } else { 0.0 };
        let width = (layout.total_width + pad * 2.0) as u32;
        let height = (layout.total_height + pad * 2.0) as u32;
        
        let mut pixmap = Pixmap::new(width, height).unwrap();
        
        let bg_color = parse_hex_color(&custom.bg_color, custom.bg_opacity);
        let mut border_color: Option<Color> = None;
        let mut border_width = 0.0;
        if custom.border_width > 0.0 {
            border_color = Some(parse_hex_color(&custom.border_color, 1.0));
            border_width = custom.border_width;
        }
        let radius = custom.radius;
        let normal_text_color = parse_hex_color(&custom.text_color, 1.0);
        let mod_text_color = normal_text_color;
        let shadow_color = parse_hex_color(&custom.shadow_color, 1.0);

        let mut current_x = 16.0 + pad;
        let y = 16.0 + pad;
        let body_h = layout.total_height;

        // 连击乘数入场动画进度（所有乘数节点共享同一时间基准）
        let mult_t = multiplier_anim_progress(multiplier_birth, std::time::Instant::now());
        let mult_alpha = ease_out_cubic(mult_t);
        let mult_scale = MULTIPLIER_SCALE_FROM + (1.0 - MULTIPLIER_SCALE_FROM) * mult_t;
        
        for node in &layout.nodes {
            let block_width = node.width;

            if node.is_multiplier {
                // 乘数 (例如 "x2") 作为一个悬浮文本显示，不绘制气泡背景
            } else if style == "3d_key" {
                // 3D 实体按键：底面（深色阴影厚度）
                let shadow_path = rounded_rect(
                    current_x, 
                    pad + 4.0, 
                    block_width, 
                    body_h - 4.0, 
                    radius
                );
                let mut shadow_paint = Paint::default();
                shadow_paint.set_color(shadow_color);
                shadow_paint.anti_alias = true;
                pixmap.fill_path(&shadow_path, &shadow_paint, FillRule::Winding, Transform::identity(), None);
                
                // 3D 实体按键：顶面（自上而下的明暗渐变增强立体感）
                let top_path = rounded_rect(
                    current_x, 
                    pad, 
                    block_width, 
                    body_h - 4.0, 
                    radius
                );
                let mut top_paint = Paint::default();
                top_paint.anti_alias = true;
                let top_light = adjust_lightness(bg_color, 0.08);
                let top_dark = adjust_lightness(bg_color, -0.08);
                if bg_color.alpha() > 0.0 {
                    if let Some(shader) = LinearGradient::new(
                        tiny_skia::Point { x: 0.0, y: pad },
                        tiny_skia::Point { x: 0.0, y: pad + body_h - 4.0 },
                        vec![
                            GradientStop::new(0.0, top_light),
                            GradientStop::new(1.0, top_dark),
                        ],
                        tiny_skia::SpreadMode::Pad,
                        Transform::identity(),
                    ) {
                        top_paint.shader = shader;
                    } else {
                        top_paint.set_color(bg_color);
                    }
                } else {
                    top_paint.set_color(bg_color);
                }
                pixmap.fill_path(&top_path, &top_paint, FillRule::Winding, Transform::identity(), None);

                // 顶面的细边框
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(border_color.unwrap_or(Color::from_rgba8(224, 224, 224, 255)));
                stroke_paint.anti_alias = true;
                let mut stroke = Stroke::default();
                stroke.width = border_width.max(1.0);
                pixmap.stroke_path(&top_path, &stroke_paint, &stroke, Transform::identity(), None);
            } else if style == "cartoon" {
                // 卡通风格：纯黑色的实心投影（新粗野主义风格）
                let shadow_path = rounded_rect(
                    current_x, 
                    pad + 6.0, 
                    block_width, 
                    body_h - 6.0, 
                    radius
                );
                let mut shadow_paint = Paint::default();
                shadow_paint.set_color(shadow_color);
                shadow_paint.anti_alias = true;
                pixmap.fill_path(&shadow_path, &shadow_paint, FillRule::Winding, Transform::identity(), None);
                
                // 卡通风格：纯白顶面
                let top_path = rounded_rect(
                    current_x + border_width / 2.0, 
                    pad + border_width / 2.0, 
                    block_width - border_width, 
                    body_h - 6.0 - border_width, 
                    radius
                );
                
                let mut top_paint = Paint::default();
                top_paint.set_color(bg_color);
                top_paint.anti_alias = true;
                pixmap.fill_path(&top_path, &top_paint, FillRule::Winding, Transform::identity(), None);

                // 卡通风格：顶面高光条（白色渐变，增强泡泡质感）
                if bg_color.alpha() > 0.5 {
                    let hl_path = rounded_rect(
                        current_x + 6.0, 
                        pad + 4.0, 
                        block_width - 12.0, 
                        (body_h - 6.0) * 0.35, 
                        radius.min((body_h - 6.0) * 0.175)
                    );
                    let mut hl_paint = Paint::default();
                    hl_paint.anti_alias = true;
                    if let Some(shader) = LinearGradient::new(
                        tiny_skia::Point { x: 0.0, y: pad + 4.0 },
                        tiny_skia::Point { x: 0.0, y: pad + 4.0 + (body_h - 6.0) * 0.35 },
                        vec![
                            GradientStop::new(0.0, Color::from_rgba8(255, 255, 255, 110)),
                            GradientStop::new(1.0, Color::from_rgba8(255, 255, 255, 0)),
                        ],
                        tiny_skia::SpreadMode::Pad,
                        Transform::identity(),
                    ) {
                        hl_paint.shader = shader;
                        pixmap.fill_path(&hl_path, &hl_paint, FillRule::Winding, Transform::identity(), None);
                    }
                }

                // 卡通风格：细黑边框
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(border_color.unwrap_or(Color::from_rgba8(0, 0, 0, 255)));
                stroke_paint.anti_alias = true;
                let mut stroke = Stroke::default();
                stroke.width = border_width;
                pixmap.stroke_path(&top_path, &stroke_paint, &stroke, Transform::identity(), None);
            } else {
                // default 风格：先绘制多层软阴影（外扩递增、alpha 递减）
                if pad > 0.0 {
                    for (expand, alpha) in SHADOW_LAYERS.iter().rev() {
                        let shadow_path = rounded_rect(
                            current_x - expand,
                            pad - expand,
                            block_width + expand * 2.0,
                            body_h + expand * 2.0,
                            radius + expand,
                        );
                        let mut sp = Paint::default();
                        sp.set_color(Color::from_rgba8(
                            (shadow_color.red() * 255.0) as u8,
                            (shadow_color.green() * 255.0) as u8,
                            (shadow_color.blue() * 255.0) as u8,
                            *alpha,
                        ));
                        sp.anti_alias = true;
                        pixmap.fill_path(&shadow_path, &sp, FillRule::Winding, Transform::identity(), None);
                    }
                }

                let path = rounded_rect(
                    current_x + border_width / 2.0, 
                    pad + border_width / 2.0, 
                    block_width - border_width, 
                    body_h - border_width, 
                    radius
                );

                if bg_color.alpha() > 0.0 {
                    let mut paint = Paint::default();
                    paint.set_color(bg_color);
                    paint.anti_alias = true;
                    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
                }

                if let Some(c) = border_color {
                    let mut paint = Paint::default();
                    paint.set_color(c);
                    paint.anti_alias = true;
                    let mut stroke = Stroke::default();
                    stroke.width = border_width;
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }

            let text_color = if node.is_modifier {
                mod_text_color
            } else {
                normal_text_color
            };
            
            // 计算文字 X 坐标，使其在气泡内居中
            // 由于字体并不总是等宽，通过 rough estimated text_width 居中。
            let text_x = current_x + (block_width - node.text_width) / 2.0;

            // 3D 实体按键和卡通气泡的顶面稍微靠上，文字也要跟着上移
            let text_y = if !node.is_multiplier && (style == "3d_key" || style == "cartoon") { y - 2.0 } else { y };

            if node.is_multiplier {
                // 根据文字颜色的亮度决定描边颜色，确保自适应任何背景
                let r = text_color.red();
                let g = text_color.green();
                let b = text_color.blue();
                let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
                
                let outline_base = if luminance > 0.5 {
                    Color::from_rgba8(0, 0, 0, 150) // 浅色文字使用深色描边
                } else {
                    Color::from_rgba8(255, 255, 255, 200) // 深色文字使用浅色描边
                };

                // 入场动画：alpha 淡入 + 字号微缩放（围绕基线锚点）
                let mult_text_color = Color::from_rgba8(
                    (text_color.red() * 255.0) as u8,
                    (text_color.green() * 255.0) as u8,
                    (text_color.blue() * 255.0) as u8,
                    (text_color.alpha() * 255.0 * mult_alpha) as u8,
                );
                let outline_color = Color::from_rgba8(
                    (outline_base.red() * 255.0) as u8,
                    (outline_base.green() * 255.0) as u8,
                    (outline_base.blue() * 255.0) as u8,
                    (outline_base.alpha() * 255.0 * mult_alpha) as u8,
                );
                let font_size = 24.0 * mult_scale;
                // 缩放时向上偏移，保持文字底部锚定，避免入场时下沉
                let scaled_y = text_y - (24.0 - font_size) * 0.7;

                self.font_renderer.draw_text(&mut pixmap, &node.text, text_x - 1.0, scaled_y, font_size, outline_color);
                self.font_renderer.draw_text(&mut pixmap, &node.text, text_x + 1.0, scaled_y, font_size, outline_color);
                self.font_renderer.draw_text(&mut pixmap, &node.text, text_x, scaled_y - 1.0, font_size, outline_color);
                self.font_renderer.draw_text(&mut pixmap, &node.text, text_x, scaled_y + 1.0, font_size, outline_color);
                self.font_renderer.draw_text(&mut pixmap, &node.text, text_x, scaled_y, font_size, mult_text_color);

                current_x += block_width + 8.0;
                continue;
            }

            let is_mouse = node.text == "LeftClick" || node.text == "RightClick" || node.text == "MiddleClick" || node.text == "ScrollUp" || node.text == "ScrollDown";
            
            if is_mouse {
                let center_x = current_x + block_width / 2.0;
                let mut center_y = pad + 28.0;
                if style == "3d_key" || style == "cartoon" {
                    center_y -= 2.0;
                }

                // 绘制鼠标身体 (圆角矩形) 放大尺寸 (32x44)
                let body = rounded_rect(center_x - 16.0, center_y - 22.0, 32.0, 44.0, 16.0);
                let mut stroke = Stroke::default();
                stroke.width = 2.5; // 边框稍微加粗
                let mut paint = Paint::default();
                paint.set_color(text_color);
                paint.anti_alias = true;
                pixmap.stroke_path(&body, &paint, &stroke, Transform::identity(), None);

                // 绘制滚轮区域分隔线
                let mut pb = tiny_skia::PathBuilder::new();
                pb.move_to(center_x, center_y - 15.0);
                pb.line_to(center_x, center_y - 4.0);
                let wheel_path = pb.finish().unwrap();
                pixmap.stroke_path(&wheel_path, &paint, &stroke, Transform::identity(), None);

                // 绘制横向分隔线
                let mut pb2 = tiny_skia::PathBuilder::new();
                pb2.move_to(center_x - 16.0, center_y + 2.0);
                pb2.line_to(center_x + 16.0, center_y + 2.0);
                let horiz_path = pb2.finish().unwrap();
                pixmap.stroke_path(&horiz_path, &paint, &stroke, Transform::identity(), None);

                if node.text == "ScrollUp" || node.text == "ScrollDown" {
                    // 绘制滚轮上/下箭头指示
                    let mut arrow_pb = tiny_skia::PathBuilder::new();
                    // 根据方向决定箭头位置和指向
                    let ay = if node.text == "ScrollUp" { center_y - 15.0 } else { center_y - 4.0 };
                    let dy = if node.text == "ScrollUp" { 4.0 } else { -4.0 }; 
                    
                    arrow_pb.move_to(center_x - 4.0, ay + dy);
                    arrow_pb.line_to(center_x, ay);
                    arrow_pb.line_to(center_x + 4.0, ay + dy);
                    
                    let mut arrow_stroke = Stroke::default();
                    arrow_stroke.width = 2.5;
                    arrow_stroke.line_cap = tiny_skia::LineCap::Round;
                    arrow_stroke.line_join = tiny_skia::LineJoin::Round;
                    pixmap.stroke_path(&arrow_pb.finish().unwrap(), &paint, &arrow_stroke, Transform::identity(), None);
                } else {
                    // 绘制按下的高亮指示圆点
                    let (ix, iy) = match node.text.as_str() {
                        "LeftClick" => (center_x - 8.0, center_y - 7.0),
                        "RightClick" => (center_x + 8.0, center_y - 7.0),
                        "MiddleClick" => (center_x, center_y - 7.0),
                        _ => (center_x, center_y),
                    };
                    
                    let mut dot_pb = tiny_skia::PathBuilder::new();
                    dot_pb.move_to(ix, iy);
                    dot_pb.line_to(ix + 0.1, iy);
                    let mut dot_stroke = Stroke::default();
                    dot_stroke.width = 6.0;
                    dot_stroke.line_cap = tiny_skia::LineCap::Round;
                    pixmap.stroke_path(&dot_pb.finish().unwrap(), &paint, &dot_stroke, Transform::identity(), None);
                }

            } else {
                // 绘制文字
                self.font_renderer.draw_text(&mut pixmap, &node.text, text_x, text_y, 24.0, text_color);
            }
            
            current_x += block_width + 8.0; // +spacing
        }
        
        pixmap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_multiplier_anim_progress_no_birth_means_done() {
        // 无出生时间戳（旧文本/动画已完成）返回 1.0
        assert_eq!(multiplier_anim_progress(None, Instant::now()), 1.0);
    }

    #[test]
    fn test_multiplier_anim_progress_boundaries() {
        let now = Instant::now();
        // 刚出生：进度接近 0
        let early = multiplier_anim_progress(Some(now), now);
        assert!(early < 0.05);
        // 超过动画时长：进度为 1.0
        let done = multiplier_anim_progress(Some(now - Duration::from_millis(200)), now);
        assert_eq!(done, 1.0);
    }

    #[test]
    fn test_multiplier_anim_progress_monotonic() {
        let now = Instant::now();
        let birth = now - Duration::from_millis(100);
        let mut prev = 0.0;
        for step in 0..=5 {
            let t = now - Duration::from_millis(100) + Duration::from_millis(step * 20);
            let p = multiplier_anim_progress(Some(birth), t.max(birth));
            assert!(p >= prev, "动画进度应单调递增");
            prev = p;
        }
    }
}
