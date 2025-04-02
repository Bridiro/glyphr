#![no_std]

pub mod fonts;
mod sdf;

type WritePixel = fn(u32, u32, u32, &mut [u32]);

#[derive(Clone, Copy)]
pub struct SdfConfig {
    pub font: fonts::Font,
    pub scale: f32,
    pub color: u32,
    pub mid_value: f32,
    pub smoothing: f32,
}

impl Default for SdfConfig {
    fn default() -> Self {
        Self {
            font: fonts::Font::default(),
            scale: 1.0,
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

    pub fn render(&mut self, phrase: &str, mut x: u32, y: u32) {
        let mut heights: [i32; 100] = [0; 100];
        let mut max_height = i32::MIN;
        for (i, c) in phrase.chars().enumerate() {
            if c != ' ' {
                let metrics = sdf::get_metrics(self, c);
                let h = ((metrics.height + metrics.ymin) as f32 * self.sdf_config.scale) as i32;
                max_height = max_height.max(h);
                heights[i] = h;
            } else {
                heights[i] = 0;
            }
        }
        for (i, c) in phrase.chars().enumerate() {
            if c != ' ' {
                sdf::render_glyph(
                    x,
                    y + (max_height - heights[i]) as u32,
                    c,
                    self,
                );
            }
            x += (sdf::advance(self, c) as f32 * self.sdf_config.scale) as u32;
        }
    }

    pub fn set_sdf_config(&mut self, config: SdfConfig) {
        self.sdf_config = config;
    }
}

#[cfg(test)]
mod tests {}
