use rusttype::{Font, Scale, point};
use tiny_skia::{Pixmap, Color, PremultipliedColorU8};

pub struct FontRenderer {
    font: Font<'static>,
}

impl FontRenderer {
    pub fn new() -> Self {
        // Load the Consolas/RobotoMono font we copied to assets
        let font_data = include_bytes!("../../assets/font.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
        Self { font }
    }

    pub fn draw_text(&self, pixmap: &mut Pixmap, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let scale = Scale::uniform(size);
        let v_metrics = self.font.v_metrics(scale);
        
        let glyphs: Vec<_> = self.font
            .layout(text, scale, point(x, y + v_metrics.ascent))
            .collect();

        let r = (color.red() * 255.0) as u8;
        let g = (color.green() * 255.0) as u8;
        let b = (color.blue() * 255.0) as u8;
        // Assume text color is mostly opaque

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = gx as i32 + bounding_box.min.x;
                    let py = gy as i32 + bounding_box.min.y;

                    if px >= 0 && px < pixmap.width() as i32 && py >= 0 && py < pixmap.height() as i32 {
                        let alpha = (v * color.alpha() * 255.0) as u8;
                        if alpha > 0 {
                            // Blending over existing color
                            let existing = pixmap.pixel(px as u32, py as u32).unwrap();
                            let out_a = alpha.saturating_add((existing.alpha() as u16 * (255 - alpha) as u16 / 255) as u8);
                            let out_r = ((r as u16 * alpha as u16 + existing.red() as u16 * (255 - alpha) as u16) / 255) as u8;
                            let out_g = ((g as u16 * alpha as u16 + existing.green() as u16 * (255 - alpha) as u16) / 255) as u8;
                            let out_b = ((b as u16 * alpha as u16 + existing.blue() as u16 * (255 - alpha) as u16) / 255) as u8;
                            
                            let width = pixmap.width();
                            if let Some(pixel) = PremultipliedColorU8::from_rgba(out_r.min(out_a), out_g.min(out_a), out_b.min(out_a), out_a) {
                                pixmap.pixels_mut()[(py as u32 * width + px as u32) as usize] = pixel;
                            }
                        }
                    }
                });
            }
        }
    }
}
