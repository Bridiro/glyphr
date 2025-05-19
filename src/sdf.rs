//! # sdf
//!
//! Contains the core logic to render SDF-based fonts with RLE decoding,
//! bilinear sampling, and blending to an output framebuffer.

#[allow(unused_imports)]
use crate::{
    Glyphr, fonts,
    glyph::{GlyphEntry, Metrics},
    utils::{ExtFloor, mix, smoothstep},
};

/// Renders a glyph at a given position.
///
/// # Panics
/// Panics if the scaled glyph size is 0.
/// 
/// # Examples
/// ```
/// # use glyphr::{sdf::render_glyph, Glyphr, SdfConfig};
/// # let mut buffer = [0u32; 100];
/// # let mut state = Glyphr::new(|_, _, _, _| {}, &mut buffer, 10, 10, SdfConfig::default());
/// render_glyph(0, 0, 'A', &mut state, 1.0);
/// ```
pub fn render_glyph(x: i32, y: i32, value: char, state: &mut Glyphr, scale: f32) {
    let sdf = &state.sdf_config.font.get_glyphs()[value as u8 as usize - 33];
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

    let distance_to_pixel = |distance: f32| match distance > state.sdf_config.mid_value {
        true => {
            (smoothstep(
                state.sdf_config.mid_value - state.sdf_config.smoothing,
                state.sdf_config.mid_value + state.sdf_config.smoothing,
                distance,
            ) * 255.0) as u8
        }
        false => 0,
    };

    for x_1 in 0..width as i32 {
        for y_1 in 0..height as i32 {
            if x_1 + x >= 0
                && x_1 + x < state.buffer.width as i32
                && y_1 + y >= 0
                && y_1 + y < state.buffer.height as i32
            {
                let sample_x = ((x_1 as f32) + 0.5) / width_f;
                let sample_y = ((y_1 as f32) + 0.5) / height_f;

                let sampled_distance = sdf_sample(&sdf, sample_x, sample_y);
                let alpha = distance_to_pixel(sampled_distance) as u32;
                if alpha > 0 {
                    let blended_color = (alpha << 24) | (state.sdf_config.color & 0x00ffffff);
                    (state.pixel_callback)(
                        (x_1 + x) as u32,
                        (y_1 + y) as u32,
                        blended_color,
                        state.buffer.buffer,
                    );
                }
            }
        }
    }
}

/// Returns the advance width for a character.
///
/// # Examples
/// ```
/// # use glyphr::{sdf::advance, Glyphr, SdfConfig};
/// # let mut buffer = [0u32; 100];
/// # let state = Glyphr::new(|_, _, _, _| {}, &mut buffer, 10, 10, SdfConfig::default());
/// let adv = advance(&state, 'A');
/// assert!(adv > 0);
/// ```
pub fn advance(state: &Glyphr, c: char) -> u32 {
    if c != ' ' {
        let sdf = &state.sdf_config.font.get_glyphs()[c as u8 as usize - 33];
        sdf.metrics.advance_width as u32
    } else {
        let sdf = &state.sdf_config.font.get_glyphs()['t' as u8 as usize - 33];
        sdf.metrics.advance_width as u32
    }
}

/// Returns the glyph metrics for the given character.
///
/// # Examples
/// ```
/// # use glyphr::{sdf::get_metrics, Glyphr, SdfConfig};
/// # let mut buffer = [0u32; 100];
/// # let state = Glyphr::new(|_, _, _, _| {}, &mut buffer, 10, 10, SdfConfig::default());
/// let m = get_metrics(&state, 'A');
/// assert!(m.advance_width > 0.0);
/// ```
pub fn get_metrics<'a>(state: &'a Glyphr, c: char) -> &'a Metrics {
    let sdf = &state.sdf_config.font.get_glyphs()[c as u8 as usize - 33];
    &sdf.metrics
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
fn sdf_sample(sdf: &GlyphEntry, x: f32, y: f32) -> f32 {
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

#[cfg(test)]
mod tests {
    use crate::fonts::Font;
    use crate::{Glyphr, SdfConfig};

    fn dummy_pixel_callback(x: u32, y: u32, color: u32, buf: &mut [u32]) {
        let idx = (y * 10 + x) as usize;
        if idx < buf.len() {
            buf[idx] = color;
        }
    }

    fn setup_dummy_state<'a>(buffer: &'a mut [u32]) -> Glyphr<'a> {
        let config = SdfConfig {
            font: Font::default(),
            align: crate::fonts::FontAlign::default(),
            px: 16,
            color: 0x112233,
            mid_value: 0.5,
            smoothing: 0.5,
        };

        Glyphr::new(dummy_pixel_callback, buffer, 10, 10, config)
    }

    #[test]
    fn test_ext_floor_behavior() {
        assert_eq!(1.9f32.floor(), 1.0);
        assert_eq!((-1.1f32).floor(), -2.0);
        assert_eq!(0.0f32.floor(), 0.0);
        assert_eq!((-0.999f32).floor(), -1.0);
    }

    #[test]
    fn test_smoothstep_behavior() {
        assert_eq!(super::smoothstep(0.0, 1.0, -1.0), 0.0);
        assert_eq!(super::smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(super::smoothstep(0.0, 1.0, 0.5), 0.5);
        assert_eq!(super::smoothstep(0.0, 1.0, 1.0), 1.0);
        assert_eq!(super::smoothstep(0.0, 1.0, 2.0), 1.0);
    }

    #[test]
    fn test_mix_behavior() {
        assert_eq!(super::mix(0.0, 10.0, 0.0), 0.0);
        assert_eq!(super::mix(0.0, 10.0, 0.5), 5.0);
        assert_eq!(super::mix(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    #[should_panic]
    fn test_render_glyph_invalid_size() {
        let mut buffer = [0u32; 100];
        let mut state = setup_dummy_state(&mut buffer);
        super::render_glyph(0, 0, ' ', &mut state, 0.0);
    }

    #[test]
    fn test_render_glyph_valid() {
        let mut buffer = [0u32; 100];
        let mut state = setup_dummy_state(&mut buffer);
        super::render_glyph(0, 0, 'A', &mut state, 1.0);
        // we expect some pixels to be written, exact values depend on sdf decoding logic
    }

    #[test]
    fn test_render_whole_string() {
        let mut buffer = [0u32; 100];
        let mut state = setup_dummy_state(&mut buffer);
        state.render("HI", 0, 0);
        // Check the buffer is not empty
        let written = state.buffer.buffer.iter().any(|&x| x != 0);
        assert!(written);
    }
}
