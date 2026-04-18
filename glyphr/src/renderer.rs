//! # renderer.rs
//!
//! Glyph rasterization for SDF and 1bpp bitmap fonts.

#[allow(unused_imports)]
use crate::{
    BitmapFormat, Callbacks, Glyphr, GlyphrError,
    rle::RleCursor,
    utils::{ExtFloor, smoothstep},
};
use glyphr_types::{Font, Glyph};

use core::cmp::{max as cmax, min as cmin};

#[inline(always)]
fn bit_is_set(bitmap: &[u8], bit_index: usize) -> bool {
    let byte = bitmap[bit_index >> 3];
    let shift = 7 - (bit_index & 7);
    ((byte >> shift) & 1) != 0
}

/// Renders a glyph at a given position.
pub fn render_glyph<P>(
    x: i32,
    y: i32,
    value: char,
    font: Font,
    state: &Glyphr,
    scale: f32,
    target: &mut Callbacks<P>,
) -> Result<(), GlyphrError>
where
    P: FnMut(u16, u16, u32) -> bool,
{
    let glyph = font.find_glyph(value).ok_or(GlyphrError::InvalidGlyph(value))?;

    match font.format {
        BitmapFormat::SDF => render_glyph_sdf(x, y, glyph, state, scale, target)?,
        BitmapFormat::Bitmap => render_glyph_bitmap(x, y, glyph, state, target)?,
    }

    Ok(())
}

#[inline(always)]
fn bilerp(p00: f32, p10: f32, p01: f32, p11: f32, wx: f32, wy: f32) -> f32 {
    let top = p00 + wx * (p10 - p00);
    let bottom = p01 + wx * (p11 - p01);
    top + wy * (bottom - top)
}

/// Renders an SDF glyph with bilinear sampling and smoothing.
fn render_glyph_sdf<P>(
    dst_x: i32,
    dst_y: i32,
    glyph: &Glyph,
    state: &Glyphr,
    scale: f32,
    target: &mut Callbacks<P>,
) -> Result<(), GlyphrError>
where
    P: FnMut(u16, u16, u32) -> bool,
{
    let out_w = (glyph.width as f32 * scale) as i32;
    let out_h = (glyph.height as f32 * scale) as i32;

    if out_w <= 0 || out_h <= 0 {
        return Ok(());
    }

    let (tgt_w_u, tgt_h_u) = target.dimensions();
    let tgt_w = i32::from(tgt_w_u);
    let tgt_h = i32::from(tgt_h_u);

    let x0 = cmax(0, dst_x);
    let y0 = cmax(0, dst_y);
    let x1 = cmin(dst_x + out_w, tgt_w);
    let y1 = cmin(dst_y + out_h, tgt_h);
    if x0 >= x1 || y0 >= y1 {
        return Ok(());
    }

    let inv255: f32 = 1.0 / 255.0;
    let src_w = glyph.width as usize;
    let src_h = glyph.height as usize;

    let inv_out_w = 1.0f32 / (out_w as f32);
    let inv_out_h = 1.0f32 / (out_h as f32);

    let cfg = state.config();
    let mid = cfg.sdf.mid_value;
    let smoothing = cfg.sdf.smoothing;
    let lo = mid - smoothing;
    let hi = mid + smoothing;

    let mut base_cur = RleCursor::new(glyph.bitmap);

    for oy in y0..y1 {
        let sy = ((oy - dst_y) as f32 + 0.5) * inv_out_h * (src_h as f32) - 0.5;
        let sy_clamped = if sy < 0.0 { 0.0 } else { sy };
        let top = (sy_clamped.floor() as isize).clamp(0, (src_h as isize) - 1) as usize;
        let wy = sy_clamped - (top as f32);
        let bottom = cmin(top + 1, src_h.saturating_sub(1));

        let row_start_top = top * src_w;
        let row_start_bottom = bottom * src_w;

        base_cur.advance_to(row_start_top);
        let mut cur_top = base_cur;
        let mut cur_bot = base_cur;
        cur_bot.advance_to(row_start_bottom);

        let mut last_left_dec_top = row_start_top;
        let mut last_left_dec_bottom = row_start_bottom;

        for ox in x0..x1 {
            let sx = ((ox - dst_x) as f32 + 0.5) * inv_out_w * (src_w as f32) - 0.5;
            let sx_clamped = if sx < 0.0 { 0.0 } else { sx };
            let left = (sx_clamped.floor() as isize).clamp(0, (src_w as isize) - 1) as usize;
            let wx = sx_clamped - (left as f32);
            let right = cmin(left + 1, src_w.saturating_sub(1));

            let li_top = row_start_top + left;
            let ri_top = row_start_top + right;
            let li_bot = row_start_bottom + left;
            let ri_bot = row_start_bottom + right;

            if li_top > last_left_dec_top {
                cur_top.advance_to(li_top);
                last_left_dec_top = li_top;
            }
            let p00 = cur_top.get(li_top);
            let p10 = cur_top.get(ri_top);

            if li_bot > last_left_dec_bottom {
                cur_bot.advance_to(li_bot);
                last_left_dec_bottom = li_bot;
            }
            let p01 = cur_bot.get(li_bot);
            let p11 = cur_bot.get(ri_bot);

            let p00f = (p00 as f32) * inv255;
            let p10f = (p10 as f32) * inv255;
            let p01f = (p01 as f32) * inv255;
            let p11f = (p11 as f32) * inv255;

            let dist = bilerp(p00f, p10f, p01f, p11f, wx, wy);

            let alpha_u8 = if dist > mid {
                (smoothstep(lo, hi, dist) * 255.0) as u8
            } else {
                0
            };

            if alpha_u8 != 0 {
                let alpha = (alpha_u8 as u32) << 24;
                let blended_color = alpha | (cfg.color & 0x00ff_ffff);
                if !target.write_pixel(ox as u16, oy as u16, blended_color) {
                    return Err(GlyphrError::InvalidTarget);
                }
            }
        }
    }

    Ok(())
}

/// Renders a Bitmap-encoded glyph (bit-packed): Y-major, early clipping, fewer repeated checks.
fn render_glyph_bitmap<P>(
    dst_x: i32,
    dst_y: i32,
    glyph: &Glyph,
    state: &Glyphr,
    target: &mut Callbacks<P>,
) -> Result<(), GlyphrError>
where
    P: FnMut(u16, u16, u32) -> bool,
{
    let w = i32::from(glyph.width);
    let h = i32::from(glyph.height);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let (tgt_w_u, tgt_h_u) = target.dimensions();
    let tgt_w = i32::from(tgt_w_u);
    let tgt_h = i32::from(tgt_h_u);

    let x0 = cmax(0, dst_x);
    let y0 = cmax(0, dst_y);
    let x1 = cmin(dst_x + w, tgt_w);
    let y1 = cmin(dst_y + h, tgt_h);
    if x0 >= x1 || y0 >= y1 {
        return Ok(());
    }

    let color = (0xffu32 << 24) | (state.config().color & 0x00ff_ffff);

    let src_w = usize::from(glyph.width);
    let src_h = usize::from(glyph.height);
    if src_w.saturating_mul(src_h) > glyph.bitmap.len().saturating_mul(8) {
        return Err(GlyphrError::OutOfBounds);
    }
    let x_src_start = (x0 - dst_x) as usize;
    for oy in y0..y1 {
        let y_src = (oy - dst_y) as usize;
        let mut x_src = x_src_start;
        let row_bit_base = y_src * src_w;

        while x_src < (x1 - dst_x) as usize {
            let bit_index = row_bit_base + x_src;
            if !bit_is_set(glyph.bitmap, bit_index) {
                x_src += 1;
                continue;
            }

            let run_start = x_src;
            x_src += 1;
            while x_src < (x1 - dst_x) as usize && bit_is_set(glyph.bitmap, row_bit_base + x_src) {
                x_src += 1;
            }

            let run_len = x_src - run_start;
            let start_x = (dst_x + run_start as i32) as u16;
            for offset in 0..run_len {
                if !target.write_pixel(start_x + offset as u16, oy as u16, color) {
                    return Err(GlyphrError::InvalidTarget);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::RleCursor;

    #[test]
    fn single_run() {
        // Stream encodes: 3 x 42
        let buf = [3u8, 42];
        let mut cur = RleCursor::new(&buf);

        assert_eq!(cur.get(0), 42);
        assert_eq!(cur.get(1), 42);
        assert_eq!(cur.get(2), 42);
    }

    #[test]
    fn multiple_runs() {
        // Stream encodes: [2 x 10, 3 x 20]
        let buf = [2, 10, 3, 20];
        let mut cur = RleCursor::new(&buf);

        assert_eq!(cur.get(0), 10);
        assert_eq!(cur.get(1), 10);
        assert_eq!(cur.get(2), 20);
        assert_eq!(cur.get(3), 20);
        assert_eq!(cur.get(4), 20);
    }

    #[test]
    fn monotonic_advance() {
        // Stream encodes: [1 x 1, 1 x 2, 1 x 3, 1 x 4]
        let buf = [1, 1, 1, 2, 1, 3, 1, 4];
        let mut cur = RleCursor::new(&buf);

        // Forward only
        for i in 0..4 {
            assert_eq!(cur.get(i), (i + 1) as u8);
        }
    }

    #[test]
    fn non_monotonic_access_forces_rescan() {
        // Stream encodes: [3 x 7, 2 x 9]
        let buf = [3, 7, 2, 9];
        let mut cur = RleCursor::new(&buf);

        // Forward is fine
        assert_eq!(cur.get(0), 7);
        assert_eq!(cur.get(3), 9);

        // Now request earlier index (non-monotonic)
        assert_eq!(cur.get(1), 7);
    }

    #[test]
    fn end_of_stream_behavior() {
        // Stream encodes: [2 x 5]
        let buf = [2, 5];
        let mut cur = RleCursor::new(&buf);

        assert_eq!(cur.get(0), 5);
        assert_eq!(cur.get(1), 5);
        // Out of bounds -> stays at last run value
        assert_eq!(cur.get(2), 0);
    }

    #[test]
    fn empty_stream() {
        let buf: [u8; 0] = [];
        let mut cur = RleCursor::new(&buf);

        // Any access should return 0
        assert_eq!(cur.get(0), 0);
        assert_eq!(cur.get(10), 0);
    }
}
