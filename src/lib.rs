#![no_std]

pub mod fonts;
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
mod tests {}
