#![no_std]

mod fonts;
mod sdf;

type WritePixel = fn(u32, u32, u32, &mut [u32]);

pub struct Buffer<'a> {
    buffer: &'a mut [u32],
    width: u32,
    height: u32,
}

pub struct Glyphr<'a> {
    current_font: &'static [fonts::GlyphEntry],
    buffer: Buffer<'a>,
    pixel_callback: WritePixel,
    mid_value: f32,
    smoothing: f32,
}

impl<'a> Glyphr<'a> {
    pub fn new(
        pixel_callback: WritePixel,
        buffer: &'a mut [u32],
        width: u32,
        height: u32,
        mid_value: f32,
        smoothing: f32,
    ) -> Self {
        Glyphr {
            current_font: &fonts::FONT_ENTRIES,
            buffer: Buffer {
                buffer,
                width,
                height,
            },
            pixel_callback,
            mid_value,
            smoothing,
        }
    }

    pub fn render(
        &mut self,
        phrase: &str,
        mut x: u32,
        y: u32,
        scale: f32,
        color: u32,
    ) {
        let mut heights: [i32; 100] = [0; 100];
        let mut max_height = i32::MIN;
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
                sdf::render_glyph(
                    x,
                    y + (max_height - heights[i]) as u32,
                    scale,
                    color,
                    c,
                    self,
                );
            }
            x += (sdf::advance(self, c) as f32 * scale) as u32;
        }
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }
}


#[cfg(test)]
mod tests {}
