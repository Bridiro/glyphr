//! # sdf.rs
//!
//! Contains the core logic to render SDF-based fonts with RLE decoding,
//! bitmap-encoded fonts rendering, bilinear sampling, and blending to an output framebuffer.

#[allow(unused_imports)]
use crate::{
    BitmapFormat, Glyphr, GlyphrError, RenderTarget,
    font::{Font, Glyph},
    utils::{ExtFloor, smoothstep},
};

use core::cmp::{max as cmax, min as cmin};

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

#[inline(always)]
fn bilerp(p00: f32, p10: f32, p01: f32, p11: f32, wx: f32, wy: f32) -> f32 {
    // Fused bilinear interpolation (no function calls to `mix` in hot loop).
    let top = p00 + wx * (p10 - p00);
    let bottom = p01 + wx * (p11 - p01);
    top + wy * (bottom - top)
}

/// A forward-only cursor to read values from an RLE [count, value] stream
/// in *decoded index* order. Works in O(1) amortized for monotonically
/// increasing target indices (our case).
#[derive(Clone, Copy)]
struct RleCursor<'a> {
    buf: &'a [u8],
    i: usize,
    run_c: usize,
    val: u8,
    dec: usize,
}

impl<'a> RleCursor<'a> {
    #[inline(always)]
    fn new(buf: &'a [u8]) -> Self {
        let mut c = Self {
            buf,
            i: 0,
            run_c: 0,
            val: 0,
            dec: 0,
        };
        c.load_next_run();
        c
    }

    #[inline(always)]
    fn load_next_run(&mut self) {
        if self.i + 1 < self.buf.len() {
            let count = self.buf[self.i] as usize;
            let value = self.buf[self.i + 1];
            self.dec += self.run_c;
            self.i += 2;
            self.run_c = count;
            self.val = value;
        } else {
            // Exhausted stream
            self.dec += self.run_c;
            self.run_c = 0;
            self.val = 0;
            self.i = self.buf.len();
        }
    }

    /// Advance forward until the run that *contains* `target_dec_idx`.
    /// `target_dec_idx` must be >= current decoded index for best performance.
    #[inline(always)]
    fn advance_to(&mut self, target_dec_idx: usize) {
        // If target is before current run start, we can't go backwards (shouldn't happen
        // in our monotonic usage). We'll just fall back to full rescan if it occurs.
        if target_dec_idx < self.dec {
            // Rare/unsafe path: rescan from the beginning (still O(N), but should not happen).
            *self = RleCursor::new(self.buf);
        }
        // Move runs forward until target is inside [dec .. dec + run_c)
        while self.run_c == 0 || target_dec_idx >= self.dec + self.run_c {
            if self.run_c == 0 && self.i >= self.buf.len() {
                // End of stream
                return;
            }
            self.load_next_run();
        }
        // Now target lies in current run
    }

    /// Get the value at `target_dec_idx`, advancing forward as needed.
    #[inline(always)]
    fn get(&mut self, target_dec_idx: usize) -> u8 {
        self.advance_to(target_dec_idx);
        self.val
    }
}

/// Renders an SDF-encoded glyph applying smoothing (Y-major scan, RLE-cursor optimized).
fn render_glyph_sdf<T: RenderTarget>(
    dst_x: i32,
    dst_y: i32,
    glyph: &Glyph,
    state: &Glyphr,
    scale: f32,
    target: &mut T,
) -> Result<(), GlyphrError> {
    // Scaled output size
    let out_w = (glyph.width as f32 * scale) as i32;
    let out_h = (glyph.height as f32 * scale) as i32;

    if out_w <= 0 || out_h <= 0 {
        return Ok(());
    }

    let (tgt_w_u, tgt_h_u) = target.dimensions();
    let tgt_w = tgt_w_u as i32;
    let tgt_h = tgt_h_u as i32;

    // Clipping to target bounds (early reject off-screen regions)
    let x0 = cmax(0, dst_x);
    let y0 = cmax(0, dst_y);
    let x1 = cmin(dst_x + out_w, tgt_w);
    let y1 = cmin(dst_y + out_h, tgt_h);
    if x0 >= x1 || y0 >= y1 {
        return Ok(());
    }

    // Precompute constants
    let inv255: f32 = 1.0 / 255.0;
    let src_w = glyph.width as usize;
    let src_h = glyph.height as usize;

    // Normalize factors to map output pixel centers to source [0,1] space.
    let inv_out_w = 1.0f32 / (out_w as f32);
    let inv_out_h = 1.0f32 / (out_h as f32);

    // SDF smoothing params (pulled out of the loop)
    let cfg = state.config();
    let mid = cfg.sdf.mid_value;
    let smoothing = cfg.sdf.smoothing;
    let lo = mid - smoothing;
    let hi = mid + smoothing;

    // A single base cursor that moves only forward as y increases.
    let mut base_cur = RleCursor::new(glyph.bitmap);

    // Iterate output rows (Y-major), cache two row cursors for top & bottom source rows.
    for oy in y0..y1 {
        // Map to source fractional row in [0, src_h)
        let sy = ((oy - dst_y) as f32 + 0.5) * inv_out_h * (src_h as f32) - 0.5;
        let sy_clamped = if sy < 0.0 { 0.0 } else { sy }; // no negative sampling
        let top = (sy_clamped.floor() as isize).clamp(0, (src_h as isize) - 1) as usize;
        let wy = sy_clamped - (top as f32);
        let bottom = cmin(top + 1, src_h.saturating_sub(1));

        // Locate the *decoded* start index for the rows we need.
        // Because oy increases, row_start_top is non-decreasing -> base_cur only moves forward.
        let row_start_top = top * src_w;
        let row_start_bottom = bottom * src_w;

        base_cur.advance_to(row_start_top);
        let mut cur_top = base_cur;
        let mut cur_bot = base_cur;
        cur_bot.advance_to(row_start_bottom);

        // We walk across X in increasing order, so decoded indices are monotonic as well.
        let mut last_left_dec_top = row_start_top;
        let mut last_left_dec_bottom = row_start_bottom;

        for ox in x0..x1 {
            // Map to source fractional column in [0, src_w)
            let sx = ((ox - dst_x) as f32 + 0.5) * inv_out_w * (src_w as f32) - 0.5;
            let sx_clamped = if sx < 0.0 { 0.0 } else { sx };
            let left = (sx_clamped.floor() as isize).clamp(0, (src_w as isize) - 1) as usize;
            let wx = sx_clamped - (left as f32);
            let right = cmin(left + 1, src_w.saturating_sub(1));

            // Global decoded indices for the four neighbors (monotone across ox)
            let li_top = row_start_top + left;
            let ri_top = row_start_top + right;
            let li_bot = row_start_bottom + left;
            let ri_bot = row_start_bottom + right;

            // Advance row cursors forward as needed (mostly +0 or +1 per step)
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

            // Normalize once via multiply (cheaper than /255.0 on MCUs)
            let p00f = (p00 as f32) * inv255;
            let p10f = (p10 as f32) * inv255;
            let p01f = (p01 as f32) * inv255;
            let p11f = (p11 as f32) * inv255;

            let dist = bilerp(p00f, p10f, p01f, p11f, wx, wy);

            // Keep your original gating behavior (only smoothstep if > mid).
            // Pull constants out of loop; smoothstep is likely cheap enough.
            let alpha_u8 = if dist > mid {
                (smoothstep(lo, hi, dist) * 255.0) as u8
            } else {
                0
            };

            if alpha_u8 != 0 {
                let alpha = (alpha_u8 as u32) << 24;
                let blended_color = alpha | (cfg.color & 0x00ff_ffff);
                if !target.write_pixel(ox as u32, oy as u32, blended_color) {
                    return Err(GlyphrError::InvalidTarget);
                }
            }
        }
    }

    Ok(())
}

/// Renders a Bitmap-encoded glyph (bit-packed): Y-major, early clipping, fewer repeated checks.
fn render_glyph_bitmap<T: RenderTarget>(
    dst_x: i32,
    dst_y: i32,
    glyph: &Glyph,
    state: &Glyphr,
    target: &mut T,
) -> Result<(), GlyphrError> {
    let w = glyph.width;
    let h = glyph.height;

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let (tgt_w_u, tgt_h_u) = target.dimensions();
    let tgt_w = tgt_w_u as i32;
    let tgt_h = tgt_h_u as i32;

    let x0 = cmax(0, dst_x);
    let y0 = cmax(0, dst_y);
    let x1 = cmin(dst_x + w, tgt_w);
    let y1 = cmin(dst_y + h, tgt_h);
    if x0 >= x1 || y0 >= y1 {
        return Ok(());
    }

    let color = (0xffu32 << 24) | (state.config().color & 0x00ff_ffff);

    for oy in y0..y1 {
        let y_src = oy - dst_y;
        for ox in x0..x1 {
            let x_src = ox - dst_x;
            if bitmap_value_at(glyph, x_src, y_src)? {
                if !target.write_pixel(ox as u32, oy as u32, color) {
                    return Err(GlyphrError::InvalidTarget);
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

/// Return a bit from a packed 1bpp bitmap.
fn bitmap_value_at(glyph: &Glyph, x: i32, y: i32) -> Result<bool, GlyphrError> {
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
