//! # sdf
//!
//! Contains the core logic to render SDF-based fonts with RLE decoding,
//! bilinear sampling, and blending to an output framebuffer.

#[allow(unused_imports)]
use crate::{
    BitmapFormat, Glyphr, GlyphrError, RenderTarget,
    font::{Font, Glyph},
    utils::{ExtFloor, mix, smoothstep},
};

/// Renders a glyph at a given position.
pub fn render_glyph<T: RenderTarget>(
    x: i32,
    y: i32,
    value: char,
    font: Font,
    state: &Glyphr,
    scale: f32,
    target: &mut T,
) -> Result<(), GlyphrError> {
    let glyph = &font.find_glyph(value)?;

    match font.format {
        BitmapFormat::SDF => render_glyph_sdf(x, y, glyph, state, scale, target)?,
        BitmapFormat::Bitmap => render_glyph_bitmap(x, y, glyph, state, target)?,
    }

    Ok(())
}

fn render_glyph_sdf<T: RenderTarget>(
    x: i32,
    y: i32,
    glyph: &Glyph,
    state: &Glyphr,
    scale: f32,
    target: &mut T,
) -> Result<(), GlyphrError> {
    let width = (glyph.width as f32 * scale) as u32;
    let height = (glyph.height as f32 * scale) as u32;

    let width_f = width as f32;
    let height_f = height as f32;

    let distance_to_pixel = |distance: f32| match distance > state.render_config.sdf.mid_value {
        true => {
            (smoothstep(
                state.render_config.sdf.mid_value - state.render_config.sdf.smoothing,
                state.render_config.sdf.mid_value + state.render_config.sdf.smoothing,
                distance,
            ) * 255.0) as u8
        }
        false => 0,
    };

    let (target_w, target_h) = target.dimensions();

    for x_1 in 0..width as i32 {
        for y_1 in 0..height as i32 {
            if x_1 + x >= 0
                && x_1 + x < target_w as i32
                && y_1 + y >= 0
                && y_1 + y < target_h as i32
            {
                let sample_x = ((x_1 as f32) + 0.5) / width_f;
                let sample_y = ((y_1 as f32) + 0.5) / height_f;

                let sampled_distance = sdf_sample(&glyph, sample_x, sample_y);
                let alpha = distance_to_pixel(sampled_distance) as u32;
                if alpha > 0 {
                    let blended_color = (alpha << 24) | (state.render_config.color & 0x00ffffff);
                    if !target.write_pixel((x_1 + x) as u32, (y_1 + y) as u32, blended_color) {
                        return Err(GlyphrError::InvalidTarget);
                    }
                }
            }
        }
    }

    Ok(())
}

fn render_glyph_bitmap<T: RenderTarget>(
    x: i32,
    y: i32,
    glyph: &Glyph,
    state: &Glyphr,
    target: &mut T,
) -> Result<(), GlyphrError> {
    let width = glyph.width;
    let height = glyph.height;

    let (target_w, target_h) = target.dimensions();

    for x_1 in 0..width as i32 {
        for y_1 in 0..height as i32 {
            if x_1 + x >= 0
                && x_1 + x < target_w as i32
                && y_1 + y >= 0
                && y_1 + y < target_h as i32
            {
                if get_bitmap_value(glyph, x_1, y_1)? {
                    let blended_color = (0xff << 24) | (state.render_config.color & 0x00ffffff);
                    if !target.write_pixel((x_1 + x) as u32, (y_1 + y) as u32, blended_color) {
                        return Err(GlyphrError::InvalidTarget);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Returns the advance width for a character.
pub fn advance(c: char, font: Font) -> Result<i32, GlyphrError> {
    Ok(font.find_glyph(c)?.advance_width)
}

/// Returns the value that would be found at a given index in a non-encoded array.
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

// This function samples the nearest 4 pixels to `x` and `y`, then does a bilinear interpolation
// and finds the average of them.
fn sdf_sample(glyph: &Glyph, x: f32, y: f32) -> f32 {
    let gx = (x * (glyph.width as f32) - 0.5).max(0.0);
    let gy = (y * (glyph.height as f32) - 0.5).max(0.0);
    let left = gx.floor() as usize;
    let top = gy.floor() as usize;
    let wx = gx - (left as f32);
    let wy = gy - (top as f32);

    let right = (left + 1).min((glyph.width - 1) as usize);
    let bottom = (top + 1).min((glyph.height - 1) as usize);

    let row_size = glyph.width as usize;
    let get_pixel = |x_1, y_1| rle_decode_at(glyph.bitmap, (row_size * y_1) + x_1 as usize);

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

fn get_bitmap_value(glyph: &Glyph, x: i32, y: i32) -> Result<bool, GlyphrError> {
    if x < 0 || y < 0 || x >= glyph.width || y >= glyph.height {
        return Err(GlyphrError::OutOfBounds);
    }

    let bit_index = y * glyph.width + x;
    let byte_index = (bit_index / 8) as usize;
    let bit_offset = (bit_index % 8) as u8;

    if byte_index >= glyph.bitmap.len() {
        return Err(GlyphrError::OutOfBounds);
    }

    let byte = glyph.bitmap[byte_index];
    let bit = (byte >> (7 - bit_offset)) & 1;

    Ok(bit == 1)
}
