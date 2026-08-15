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

    pub fn draw(&self, keys: &[String], style: &str, custom: &crate::state_machine::CustomStyle, multiplier_birth: Option<std::time::Instant>, scale: f32) -> Pixmap {
        let items = [crate::state_machine::RenderHistoryItem {
            keys: keys.to_vec(),
            multiplier_birth,
            alpha: 1.0,
        }];
        self.draw_history(&items, style, custom, scale)
    }

    pub fn draw_history(&self, items: &[crate::state_machine::RenderHistoryItem], style: &str, custom: &crate::state_machine::CustomStyle, scale: f32) -> Pixmap {
        if items.is_empty() {
            return Pixmap::new(1, 1).unwrap();
        }

        let layouts: Vec<BubbleLayout> = items.iter().map(|it| BubbleLayout::build(&it.keys)).collect();
        let pad = if style == "default" { SHADOW_PADDING } else { 0.0 };

        let max_w = layouts.iter().map(|l| l.total_width).fold(0.0f32, f32::max);
        let total_h = layouts.iter().map(|l| l.total_height).sum::<f32>() + ((layouts.len().saturating_sub(1)) as f32) * 8.0;

        let width = (max_w + pad * 2.0).ceil() as u32;
        let height = (total_h + pad * 2.0).ceil() as u32;

        let mut pixmap = Pixmap::new(width.max(1), height.max(1)).unwrap();

        let mut current_y = pad;
        for (item, layout) in items.iter().zip(layouts.iter()) {
            self.draw_row(&mut pixmap, layout, style, custom, item.multiplier_birth, pad, current_y, item.alpha);
            current_y += layout.total_height + 8.0;
        }

        if (scale - 1.0).abs() < 1e-4 {
            pixmap
        } else {
            let mut scaled_pixmap = Pixmap::new(width.max(1), height.max(1)).unwrap();
            let cx = width as f32 / 2.0;
            let cy = height as f32 / 2.0;
            let transform = Transform::from_translate(-cx, -cy)
                .post_scale(scale, scale)
                .post_translate(cx, cy);
            let mut paint = tiny_skia::PixmapPaint::default();
            paint.quality = tiny_skia::FilterQuality::Bilinear;
            scaled_pixmap.draw_pixmap(0, 0, pixmap.as_ref(), &paint, transform, None);
            scaled_pixmap
        }
    }

    fn draw_row(
        &self,
        pixmap: &mut Pixmap,
        layout: &BubbleLayout,
        style: &str,
        custom: &crate::state_machine::CustomStyle,
        multiplier_birth: Option<std::time::Instant>,
        row_x: f32,
        row_y: f32,
        row_alpha: f32,
    ) {
        let bg_color = parse_hex_color(&custom.bg_color, custom.bg_opacity * row_alpha);
        let mut border_color: Option<Color> = None;
        let mut border_width = 0.0;
        if custom.border_width > 0.0 {
            border_color = Some(parse_hex_color(&custom.border_color, row_alpha));
            border_width = custom.border_width;
        }
        let radius = custom.radius;
        let normal_text_color = parse_hex_color(&custom.text_color, row_alpha);
        let mod_text_color = normal_text_color;
        let shadow_color = parse_hex_color(&custom.shadow_color, row_alpha);

        let mut current_x = row_x + 16.0;
        let y = row_y + 16.0;
        let body_h = layout.total_height;

        let mult_t = multiplier_anim_progress(multiplier_birth, std::time::Instant::now());
        let mult_alpha = ease_out_cubic(mult_t) * row_alpha;
        let mult_scale = MULTIPLIER_SCALE_FROM + (1.0 - MULTIPLIER_SCALE_FROM) * mult_t;

        for node in &layout.nodes {
            let block_width = node.width;

            if node.is_multiplier {
                // 乘数胶囊
            } else if style == "3d_key" {
                // 3D 实体按键：底面
                let shadow_path = rounded_rect(
                    current_x, 
                    row_y + 4.0, 
                    block_width, 
                    body_h - 4.0, 
                    radius
                );
                let mut shadow_paint = Paint::default();
                shadow_paint.set_color(shadow_color);
                shadow_paint.anti_alias = true;
                pixmap.fill_path(&shadow_path, &shadow_paint, FillRule::Winding, Transform::identity(), None);
                
                // 3D 实体按键：顶面
                let top_path = rounded_rect(
                    current_x, 
                    row_y, 
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
                        tiny_skia::Point { x: 0.0, y: row_y },
                        tiny_skia::Point { x: 0.0, y: row_y + body_h - 4.0 },
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

                let mut stroke_paint = Paint::default();
                let def_b = Color::from_rgba8(224, 224, 224, (255.0 * row_alpha) as u8);
                stroke_paint.set_color(border_color.unwrap_or(def_b));
                stroke_paint.anti_alias = true;
                let mut stroke = Stroke::default();
                stroke.width = border_width.max(1.0);
                pixmap.stroke_path(&top_path, &stroke_paint, &stroke, Transform::identity(), None);
            } else if style == "cartoon" {
                // 卡通风格
                let shadow_path = rounded_rect(
                    current_x, 
                    row_y + 6.0, 
                    block_width, 
                    body_h - 6.0, 
                    radius
                );
                let mut shadow_paint = Paint::default();
                shadow_paint.set_color(shadow_color);
                shadow_paint.anti_alias = true;
                pixmap.fill_path(&shadow_path, &shadow_paint, FillRule::Winding, Transform::identity(), None);
                
                let top_path = rounded_rect(
                    current_x + border_width / 2.0, 
                    row_y + border_width / 2.0, 
                    block_width - border_width, 
                    body_h - 6.0 - border_width, 
                    radius
                );
                
                let mut top_paint = Paint::default();
                top_paint.set_color(bg_color);
                top_paint.anti_alias = true;
                pixmap.fill_path(&top_path, &top_paint, FillRule::Winding, Transform::identity(), None);

                if bg_color.alpha() > 0.5 * row_alpha {
                    let hl_path = rounded_rect(
                        current_x + 6.0, 
                        row_y + 4.0, 
                        block_width - 12.0, 
                        (body_h - 6.0) * 0.35, 
                        radius.min((body_h - 6.0) * 0.175)
                    );
                    let mut hl_paint = Paint::default();
                    hl_paint.anti_alias = true;
                    if let Some(shader) = LinearGradient::new(
                        tiny_skia::Point { x: 0.0, y: row_y + 4.0 },
                        tiny_skia::Point { x: 0.0, y: row_y + 4.0 + (body_h - 6.0) * 0.35 },
                        vec![
                            GradientStop::new(0.0, Color::from_rgba8(255, 255, 255, (110.0 * row_alpha) as u8)),
                            GradientStop::new(1.0, Color::from_rgba8(255, 255, 255, 0)),
                        ],
                        tiny_skia::SpreadMode::Pad,
                        Transform::identity(),
                    ) {
                        hl_paint.shader = shader;
                        pixmap.fill_path(&hl_path, &hl_paint, FillRule::Winding, Transform::identity(), None);
                    }
                }

                let mut stroke_paint = Paint::default();
                let def_b = Color::from_rgba8(0, 0, 0, (255.0 * row_alpha) as u8);
                stroke_paint.set_color(border_color.unwrap_or(def_b));
                stroke_paint.anti_alias = true;
                let mut stroke = Stroke::default();
                stroke.width = border_width;
                pixmap.stroke_path(&top_path, &stroke_paint, &stroke, Transform::identity(), None);
            } else {
                // default 风格软阴影
                if style == "default" {
                    for (expand, alpha) in SHADOW_LAYERS.iter().rev() {
                        let shadow_path = rounded_rect(
                            current_x - expand,
                            row_y - expand,
                            block_width + expand * 2.0,
                            body_h + expand * 2.0,
                            radius + expand,
                        );
                        let mut sp = Paint::default();
                        sp.set_color(Color::from_rgba8(
                            (shadow_color.red() * 255.0) as u8,
                            (shadow_color.green() * 255.0) as u8,
                            (shadow_color.blue() * 255.0) as u8,
                            ((*alpha as f32) * row_alpha) as u8,
                        ));
                        sp.anti_alias = true;
                        pixmap.fill_path(&shadow_path, &sp, FillRule::Winding, Transform::identity(), None);
                    }
                }

                let path = rounded_rect(
                    current_x + border_width / 2.0, 
                    row_y + border_width / 2.0, 
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
            
            let text_x = current_x + (block_width - node.text_width) / 2.0;
            let text_y = if !node.is_multiplier && (style == "3d_key" || style == "cartoon") { y - 2.0 } else { y };

            if node.is_multiplier {
                let badge_h = 30.0;
                let badge_w = block_width;
                let badge_radius = 8.0;
                let badge_cx = current_x + badge_w / 2.0;
                let badge_cy = row_y + body_h / 2.0;

                let scaled_w = badge_w * mult_scale;
                let scaled_h = badge_h * mult_scale;
                let badge_x = badge_cx - scaled_w / 2.0;
                let badge_y = badge_cy - scaled_h / 2.0;

                let shadow_path = rounded_rect(badge_x, badge_y + 1.5, scaled_w, scaled_h, badge_radius);
                let mut sp = Paint::default();
                sp.set_color(Color::from_rgba8(0, 0, 0, (90.0 * mult_alpha) as u8));
                sp.anti_alias = true;
                pixmap.fill_path(&shadow_path, &sp, FillRule::Winding, Transform::identity(), None);

                let badge_path = rounded_rect(badge_x, badge_y, scaled_w, scaled_h, badge_radius);
                let mut bp = Paint::default();
                let base_alpha = (0.90 * mult_alpha).clamp(0.0, 1.0);
                let bg_badge = Color::from_rgba8(26, 27, 33, (255.0 * base_alpha) as u8);
                bp.set_color(bg_badge);
                bp.anti_alias = true;
                pixmap.fill_path(&badge_path, &bp, FillRule::Winding, Transform::identity(), None);

                let mut stroke_p = Paint::default();
                let border_alpha = (0.35 * mult_alpha).clamp(0.0, 1.0);
                stroke_p.set_color(Color::from_rgba8(255, 255, 255, (255.0 * border_alpha) as u8));
                stroke_p.anti_alias = true;
                let mut stroke = Stroke::default();
                stroke.width = 1.0;
                pixmap.stroke_path(&badge_path, &stroke_p, &stroke, Transform::identity(), None);

                let font_size = 18.0 * mult_scale;
                let text_x = badge_cx - (node.text_width * mult_scale) / 2.0;
                let text_y = badge_cy - 9.0 * mult_scale;

                let txt_color = Color::from_rgba8(
                    245, 245, 250,
                    (255.0 * mult_alpha).clamp(0.0, 255.0) as u8,
                );
                self.font_renderer.draw_text(pixmap, &node.text, text_x, text_y, font_size, txt_color);

                current_x += block_width + 8.0;
                continue;
            }

            let is_mouse = node.text == "LeftClick" || node.text == "RightClick" || node.text == "MiddleClick" || node.text == "ScrollUp" || node.text == "ScrollDown";
            
            if is_mouse {
                let center_x = current_x + block_width / 2.0;
                let mut center_y = row_y + 28.0;
                if style == "3d_key" || style == "cartoon" {
                    center_y -= 2.0;
                }

                let body = rounded_rect(center_x - 16.0, center_y - 22.0, 32.0, 44.0, 16.0);
                let mut stroke = Stroke::default();
                stroke.width = 2.5;
                let mut paint = Paint::default();
                paint.set_color(text_color);
                paint.anti_alias = true;
                pixmap.stroke_path(&body, &paint, &stroke, Transform::identity(), None);

                let mut pb = tiny_skia::PathBuilder::new();
                pb.move_to(center_x, center_y - 15.0);
                pb.line_to(center_x, center_y - 4.0);
                let wheel_path = pb.finish().unwrap();
                pixmap.stroke_path(&wheel_path, &paint, &stroke, Transform::identity(), None);

                let mut pb2 = tiny_skia::PathBuilder::new();
                pb2.move_to(center_x - 16.0, center_y + 2.0);
                pb2.line_to(center_x + 16.0, center_y + 2.0);
                let horiz_path = pb2.finish().unwrap();
                pixmap.stroke_path(&horiz_path, &paint, &stroke, Transform::identity(), None);

                if node.text == "ScrollUp" || node.text == "ScrollDown" {
                    let mut arrow_pb = tiny_skia::PathBuilder::new();
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
                self.font_renderer.draw_text(pixmap, &node.text, text_x, text_y, 24.0, text_color);
            }
            
            current_x += block_width + 8.0;
        }
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
