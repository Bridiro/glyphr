use super::{Glyphr, fonts};

pub trait ExtFloor {
    #[allow(unused)]
    fn floor(self) -> f32;
}

impl ExtFloor for f32 {
    fn floor(self) -> f32 {
        let mut xi = self as i32;
        if self < 0.0 && self != xi as f32 {
            xi -= 1;
        }
        xi as f32
    }
}

impl<'a> Glyphr<'a> {
    pub fn new(pixel_callback: super::WritePixel, buffer: &'a mut [u32], width: u32, height: u32) -> Self {
        Glyphr {
            current_font: &fonts::FONT_POPPINS_ENTRIES,
            buffer: super::Buffer {
                buffer,
                width,
                height,
            },
            pixel_callback,
        }
    }

    pub fn render(
        &mut self,
        phrase: &str,
        x: u32,
        y: u32,
        scale: f32,
        mid_value: f32,
        smoothing: f32,
        color: u32,
    ) {
        for c in phrase.chars() {
            render_glyph(x, y, scale, mid_value, smoothing, color, c, self);
        }
    }

    pub fn get_buffer(&self) -> &super::Buffer {
        &self.buffer
    }
}

fn render_glyph(
    x: u32,
    y: u32,
    scale: f32,
    mid_value: f32,
    smoothing: f32,
    color: u32,
    value: char,
    state: &mut Glyphr,
) {
    let sdf = &state.current_font[value as u8 as usize - 33];
    let width = (sdf.metrics.width as f32 * scale) as u32;
    let height = (sdf.metrics.height as f32 * scale) as u32;
    if width <= 0 || height <= 0 {
        panic!(
            "Scaling of {:?} returns an image size of {:?}, which is impossible to render",
            scale,
            (width, height)
        );
    }

    let width_f = width as f32;
    let height_f = height as f32;

    let distance_to_pixel = |distance: f32| match distance > mid_value {
        true => (smoothstep(mid_value - smoothing, mid_value + smoothing, distance) * 255.0) as u8,
        false => 0,
    };

    for x_1 in 0..width {
        for y_1 in 0..height {
            let sample_x = ((x_1 as f32) + 0.5) / width_f;
            let sample_y = ((y_1 as f32) + 0.5) / height_f;

            let sampled_distance = sdf_sample(&sdf, sample_x, sample_y);
            let alpha = distance_to_pixel(sampled_distance);
            if alpha > 0 {
                let blended_color = blend_pixel(color, state.buffer.buffer[((y_1+y)*state.buffer.width+x_1+x) as usize], alpha);
                (state.pixel_callback)(x_1 + x, y_1 + y, blended_color, state.buffer.buffer);
            }
        }
    }
}

fn rle_decode_at(buffer: &[u8], index: usize) -> u8 {
    let mut i = 0;
    let mut decoded_index = 0;
    while i < buffer.len() {
        let count = buffer[i] as usize;
        let value = buffer[i + 1];
        if decoded_index + count > index {
            return value;
        }
        decoded_index += count;
        i += 2;
    }
    0
}

fn sdf_sample(sdf: &fonts::GlyphEntry, x: f32, y: f32) -> f32 {
    let gx = (x * (sdf.metrics.width as f32) - 0.5).max(0.0);
    let gy = (y * (sdf.metrics.height as f32) - 0.5).max(0.0);
    let left = gx.floor() as usize;
    let top = gy.floor() as usize;
    let wx = gx - (left as f32);
    let wy = gy - (top as f32);

    let right = (left + 1).min((sdf.metrics.width - 1) as usize);
    let bottom = (top + 1).min((sdf.metrics.height - 1) as usize);

    let row_size = sdf.metrics.width as usize;
    let get_pixel = |x_1, y_1| rle_decode_at(sdf.glyph, (row_size * y_1) + x_1 as usize);

    let p00 = get_pixel(left, top);
    let p10 = get_pixel(right, top);
    let p01 = get_pixel(left, bottom);
    let p11 = get_pixel(right, bottom);

    mix(
        mix(p00 as f32 / 255.0, p10 as f32 / 255.0, wx),
        mix(p01 as f32 / 255.0, p11 as f32 / 255.0, wx),
        wy,
    )
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(v1: f32, v2: f32, weight: f32) -> f32 {
    v1 + (v2 - v1) * weight
}

fn blend_pixel(fg: u32, bg: u32, alpha: u8) -> u32 {
    let alpha_f = alpha as f32 / 255.0;

    let fg_r = (fg >> 16) & 0xFF;
    let fg_g = (fg >> 8) & 0xFF;
    let fg_b = fg & 0xFF;

    let bg_r = (bg >> 16) & 0xFF;
    let bg_g = (bg >> 8) & 0xFF;
    let bg_b = bg & 0xFF;

    let blended_r = ((fg_r as f32 * alpha_f) + (bg_r as f32 * (1.0 - alpha_f))) as u8;
    let blended_g = ((fg_g as f32 * alpha_f) + (bg_g as f32 * (1.0 - alpha_f))) as u8;
    let blended_b = ((fg_b as f32 * alpha_f) + (bg_b as f32 * (1.0 - alpha_f))) as u8;

    (255 << 24) | ((blended_r as u32) << 16) | ((blended_g as u32) << 8) | (blended_b as u32)
}
