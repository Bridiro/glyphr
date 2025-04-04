#![no_std]

pub mod fonts;
mod utils;
mod sdf;

type WritePixel = fn(u32, u32, u32, &mut [u32]);

#[derive(Clone, Copy)]
pub struct SdfConfig {
    pub font: fonts::Font,
    pub align: fonts::FontAlign,
    pub px: u32,
    pub color: u32,
    pub mid_value: f32,
    pub smoothing: f32,
}

impl Default for SdfConfig {
    fn default() -> Self {
        Self {
            font: fonts::Font::default(),
            align: fonts::FontAlign::default(),
            px: fonts::Font::default().get_glyphs()[0].px,
            color: 0x000000,
            mid_value: 0.5,
            smoothing: 0.5,
        }
    }
}

pub struct Buffer<'a> {
    buffer: &'a mut [u32],
    width: u32,
    height: u32,
}

pub struct Glyphr<'a> {
    buffer: Buffer<'a>,
    pixel_callback: WritePixel,
    sdf_config: SdfConfig,
}

impl<'a> Glyphr<'a> {
    pub fn new(
        pixel_callback: WritePixel,
        buffer: &'a mut [u32],
        width: u32,
        height: u32,
        sdf_config: SdfConfig,
    ) -> Self {
        Glyphr {
            buffer: Buffer {
                buffer,
                width,
                height,
            },
            pixel_callback,
            sdf_config,
        }
    }

    pub fn set_sdf_config(&mut self, config: SdfConfig) {
        self.sdf_config = config;
    }

    pub fn set_font(&mut self, font: fonts::Font) {
        self.sdf_config.font = font;
    }

    pub fn set_font_align(&mut self, align: fonts::FontAlign) {
        self.sdf_config.align = align;
    }

    pub fn set_scale(&mut self, px: u32) {
        self.sdf_config.px = px;
    }

    pub fn set_color(&mut self, color: u32) {
        self.sdf_config.color = color;
    }

    pub fn set_smoothing(&mut self, smoothing: f32) {
        self.sdf_config.smoothing = smoothing;
    }

    pub fn set_mid_value(&mut self, mid_value: f32) {
        self.sdf_config.mid_value = mid_value;
    }

    pub fn render(&mut self, phrase: &str, mut x: i32, y: i32) {
        let mut heights: [i32; 100] = [0; 100];
        let mut max_height = i32::MIN;
        let scale = self.sdf_config.px as f32 / self.sdf_config.font.get_glyphs()[0].px as f32;

        use fonts::FontAlign;
        match self.sdf_config.align {
            FontAlign::Center => x -= phrase_length(self, phrase) / 2,
            FontAlign::Right => x -= phrase_length(self, phrase),
            _ => {}
        }
        for (i, c) in phrase.chars().enumerate() {
            if c != ' ' {
                let metrics = sdf::get_metrics(self, c);
                let h = ((metrics.height + metrics.ymin) as f32 * scale) as i32;
                max_height = max_height.max(h);
                heights[i] = h;
            } else {
                heights[i] = 0;
            }
        }
        for (i, c) in phrase.chars().enumerate() {
            if c != ' ' {
                sdf::render_glyph(x, y + (max_height - heights[i]) as i32, c, self, scale);
            }
            x += (sdf::advance(self, c) as f32 * scale) as i32;
        }
    }
}

fn phrase_length(state: &mut Glyphr, phrase: &str) -> i32 {
    let scale = state.sdf_config.px as f32 / state.sdf_config.font.get_glyphs()[0].px as f32;
    let mut tot = 0;
    for c in phrase.chars() {
        tot += (sdf::advance(state, c) as f32 * scale) as i32;
    }
    tot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::{Font, FontAlign};

    fn dummy_pixel_callback(x: u32, y: u32, color: u32, buf: &mut [u32]) {
        let idx = (y * 4 + x) as usize;
        if idx < buf.len() {
            buf[idx] = color;
        }
    }

    fn make_test_config() -> SdfConfig {
        let font = Font::default();
        SdfConfig {
            font,
            align: FontAlign::Left,
            px: 24,
            color: 0xAABBCC,
            mid_value: 0.4,
            smoothing: 0.2,
        }
    }

    #[test]
    fn test_sdf_config_default_values() {
        let cfg = SdfConfig::default();
        assert_eq!(cfg.color, 0x000000);
        assert!(cfg.mid_value > 0.0 && cfg.mid_value <= 1.0);
        assert!(cfg.smoothing > 0.0 && cfg.smoothing <= 1.0);
        let g = cfg.font.get_glyphs();
        assert!(!g.is_empty());
    }

    #[test]
    fn test_sdf_config_custom_values() {
        let cfg = make_test_config();
        assert_eq!(cfg.color, 0xAABBCC);
        assert_eq!(cfg.px, 24);
        assert_eq!(cfg.mid_value, 0.4);
        assert_eq!(cfg.smoothing, 0.2);
    }

    #[test]
    fn test_glyphr_new_initializes_correctly() {
        let mut buffer = [0u32; 16];
        let config = make_test_config();

        let glyphr = Glyphr::new(dummy_pixel_callback, &mut buffer, 4, 4, config);

        assert_eq!(glyphr.buffer.width, 4);
        assert_eq!(glyphr.buffer.height, 4);
        assert_eq!(glyphr.sdf_config.color, 0xAABBCC);
    }

    #[test]
    fn test_pixel_callback_writes_color() {
        let mut buffer = [0u32; 16];
        let callback = |x, y, color, buf: &mut [u32]| {
            let idx = (y * 4 + x) as usize;
            buf[idx] = color;
        };

        callback(2, 1, 0x123456, &mut buffer);

        let idx = 1 * 4 + 2;
        assert_eq!(buffer[idx], 0x123456);
    }

    #[test]
    fn test_out_of_bounds_pixel_callback_does_not_crash() {
        let mut buffer = [0u32; 16];
        dummy_pixel_callback(10, 10, 0xFFFFFF, &mut buffer);
        assert!(buffer.iter().all(|&b| b == 0)); // Should be unchanged
    }

    #[test]
    fn test_glyphr_multiple_renders_dont_corrupt() {
        let mut buffer = [0u32; 16];
        let mut glyphr = Glyphr::new(
            dummy_pixel_callback,
            &mut buffer,
            4,
            4,
            SdfConfig::default(),
        );

        glyphr.render("H", 0, 0);

        let modified = buffer.iter().any(|&c| c != 0);
        assert!(modified);
    }
}
