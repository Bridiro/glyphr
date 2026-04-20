//! # renderer.rs
//!
//! Glyph rasterization for SDF and 1bpp bitmap fonts.

#[allow(unused_imports)]
use crate::{
    BitmapFormat, BulkCallbacks, Callbacks, Glyphr, GlyphrError,
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
    let glyph = font
        .find_glyph(value)
        .ok_or(GlyphrError::InvalidGlyph(value))?;

    match font.format {
        BitmapFormat::SDF => render_glyph_sdf(x, y, glyph, state, scale, target)?,
        BitmapFormat::Bitmap => render_glyph_bitmap(x, y, glyph, state, target)?,
    }

    Ok(())
}

/// Renders a glyph and emits a full ARGB tile in a single callback.
pub fn render_glyph_bulk<P>(
    x: i32,
    y: i32,
    value: char,
    font: Font,
    state: &Glyphr,
    scale: f32,
    target: &mut BulkCallbacks<'_, P>,
) -> Result<(), GlyphrError>
where
    P: FnMut(i32, i32, u16, u16, &[u32]) -> bool,
{
    let glyph = font
        .find_glyph(value)
        .ok_or(GlyphrError::InvalidGlyph(value))?;

    match font.format {
        BitmapFormat::SDF => render_glyph_sdf_bulk(x, y, glyph, state, scale, target)?,
        BitmapFormat::Bitmap => render_glyph_bitmap_bulk(x, y, glyph, state, target)?,
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

/// Renders an SDF glyph into a temporary ARGB tile and emits it in one callback.
fn render_glyph_sdf_bulk<P>(
    dst_x: i32,
    dst_y: i32,
    glyph: &Glyph,
    state: &Glyphr,
    scale: f32,
    target: &mut BulkCallbacks<'_, P>,
) -> Result<(), GlyphrError>
where
    P: FnMut(i32, i32, u16, u16, &[u32]) -> bool,
{
    let out_w = (glyph.width as f32 * scale) as i32;
    let out_h = (glyph.height as f32 * scale) as i32;

    if out_w <= 0 || out_h <= 0 {
        return Ok(());
    }

    let out_w_usize = out_w as usize;
    let out_h_usize = out_h as usize;
    let needed = out_w_usize.saturating_mul(out_h_usize);
    let available = target.scratch_capacity();
    if needed > available {
        return Err(GlyphrError::BufferTooSmall { needed, available });
    }

    let (tgt_w_u, tgt_h_u) = target.dimensions();
    let tgt_w = i32::from(tgt_w_u);
    let tgt_h = i32::from(tgt_h_u);
    if dst_x >= tgt_w || dst_y >= tgt_h || dst_x + out_w <= 0 || dst_y + out_h <= 0 {
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

    {
        let scratch = target.scratch_mut();
        let tile = &mut scratch[..needed];
        tile.fill(0);

        let mut base_cur = RleCursor::new(glyph.bitmap);

        for gy in 0..out_h_usize {
            let sy = (gy as f32 + 0.5) * inv_out_h * (src_h as f32) - 0.5;
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

            for gx in 0..out_w_usize {
                let sx = (gx as f32 + 0.5) * inv_out_w * (src_w as f32) - 0.5;
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
                if dist <= mid {
                    continue;
                }

                let alpha_u8 = (smoothstep(lo, hi, dist) * 255.0) as u8;
                if alpha_u8 == 0 {
                    continue;
                }

                let alpha = (alpha_u8 as u32) << 24;
                tile[gy * out_w_usize + gx] = alpha | (cfg.color & 0x00ff_ffff);
            }
        }
    }

    if !target.emit_from_scratch(dst_x, dst_y, out_w as u16, out_h as u16, needed) {
        return Err(GlyphrError::InvalidTarget);
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

/// Renders a Bitmap glyph into a temporary ARGB tile and emits it in one callback.
fn render_glyph_bitmap_bulk<P>(
    dst_x: i32,
    dst_y: i32,
    glyph: &Glyph,
    state: &Glyphr,
    target: &mut BulkCallbacks<'_, P>,
) -> Result<(), GlyphrError>
where
    P: FnMut(i32, i32, u16, u16, &[u32]) -> bool,
{
    let out_w = usize::from(glyph.width);
    let out_h = usize::from(glyph.height);
    let needed = out_w.saturating_mul(out_h);
    let available = target.scratch_capacity();
    if needed > available {
        return Err(GlyphrError::BufferTooSmall { needed, available });
    }

    if needed == 0 {
        return Ok(());
    }

    let (tgt_w_u, tgt_h_u) = target.dimensions();
    let tgt_w = i32::from(tgt_w_u);
    let tgt_h = i32::from(tgt_h_u);
    let w_i32 = out_w as i32;
    let h_i32 = out_h as i32;
    if dst_x >= tgt_w || dst_y >= tgt_h || dst_x + w_i32 <= 0 || dst_y + h_i32 <= 0 {
        return Ok(());
    }

    let src_w = out_w;
    let src_h = out_h;
    if src_w.saturating_mul(src_h) > glyph.bitmap.len().saturating_mul(8) {
        return Err(GlyphrError::OutOfBounds);
    }

    let color = (0xffu32 << 24) | (state.config().color & 0x00ff_ffff);

    {
        let scratch = target.scratch_mut();
        let tile = &mut scratch[..needed];
        tile.fill(0);

        for gy in 0..src_h {
            let row_bit_base = gy * src_w;
            for gx in 0..src_w {
                if bit_is_set(glyph.bitmap, row_bit_base + gx) {
                    tile[gy * src_w + gx] = color;
                }
            }
        }
    }

    if !target.emit_from_scratch(dst_x, dst_y, glyph.width, glyph.height, needed) {
        return Err(GlyphrError::InvalidTarget);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{bit_is_set, render_glyph, render_glyph_bulk};
    use crate::{BulkCallbacks, Callbacks, Glyphr, GlyphrError, RenderConfig};
    use glyphr_types::{BitmapFormat, Font, Glyph};

    fn make_bitmap_font<'a>(glyphs: &'a [Glyph<'a>]) -> Font<'a> {
        Font {
            glyphs,
            size: 16,
            ascent: 2,
            descent: 0,
            line_gap: 0,
            format: BitmapFormat::Bitmap,
        }
    }

    #[test]
    fn bit_is_set_reads_expected_msb_order() {
        let bitmap = [0b1011_0000u8];
        assert!(bit_is_set(&bitmap, 0));
        assert!(!bit_is_set(&bitmap, 1));
        assert!(bit_is_set(&bitmap, 2));
        assert!(bit_is_set(&bitmap, 3));
    }

    #[test]
    fn render_glyph_bitmap_applies_clipping() {
        let glyph_bitmap = [0b1001_0000u8];
        let glyphs = [Glyph {
            character: 'A',
            bitmap: &glyph_bitmap,
            width: 2,
            height: 2,
            xmin: 0,
            ymin: 0,
            advance_width: 2,
        }];
        let font = make_bitmap_font(&glyphs);
        let renderer = Glyphr::new();

        let mut calls = [(u16::MAX, u16::MAX, 0u32); 4];
        let mut count = 0usize;
        let mut target = Callbacks::new(2, 2, |x, y, color| {
            calls[count] = (x, y, color);
            count += 1;
            true
        });

        render_glyph(-1, 0, 'A', font, &renderer, 1.0, &mut target).unwrap();

        assert_eq!(count, 1);
        assert_eq!(calls[0].0, 0);
        assert_eq!(calls[0].1, 1);
    }

    #[test]
    fn render_glyph_bulk_bitmap_emits_one_tile() {
        let glyph_bitmap = [0b1111_0000u8];
        let glyphs = [Glyph {
            character: 'B',
            bitmap: &glyph_bitmap,
            width: 2,
            height: 2,
            xmin: 0,
            ymin: 0,
            advance_width: 2,
        }];
        let font = make_bitmap_font(&glyphs);
        let renderer = Glyphr::with_config(RenderConfig::default().with_color(0x0011_2233));

        let mut scratch = [0u32; 16];
        let mut calls = 0usize;
        let mut last_x = 0i32;
        let mut last_y = 0i32;
        let mut last_w = 0u16;
        let mut last_h = 0u16;
        let mut last_pixels = [0u32; 4];

        let mut target = BulkCallbacks::new(32, 32, &mut scratch, |x, y, w, h, pixels| {
            calls += 1;
            last_x = x;
            last_y = y;
            last_w = w;
            last_h = h;
            last_pixels.copy_from_slice(pixels);
            true
        });

        render_glyph_bulk(5, 7, 'B', font, &renderer, 1.0, &mut target).unwrap();

        assert_eq!(calls, 1);
        assert_eq!(last_x, 5);
        assert_eq!(last_y, 7);
        assert_eq!(last_w, 2);
        assert_eq!(last_h, 2);
        assert_eq!(last_pixels, [0xff11_2233; 4]);
    }

    #[test]
    fn render_glyph_bulk_propagates_invalid_target() {
        let glyph_bitmap = [0b1000_0000u8];
        let glyphs = [Glyph {
            character: 'C',
            bitmap: &glyph_bitmap,
            width: 1,
            height: 1,
            xmin: 0,
            ymin: 0,
            advance_width: 1,
        }];
        let font = make_bitmap_font(&glyphs);
        let renderer = Glyphr::new();
        let mut scratch = [0u32; 4];
        let mut target = BulkCallbacks::new(4, 4, &mut scratch, |_x, _y, _w, _h, _pixels| false);

        let result = render_glyph_bulk(0, 0, 'C', font, &renderer, 1.0, &mut target);
        assert!(matches!(result, Err(GlyphrError::InvalidTarget)));
    }

    #[test]
    fn render_glyph_returns_invalid_glyph_for_missing_char() {
        let glyph_bitmap = [0b1000_0000u8];
        let glyphs = [Glyph {
            character: 'A',
            bitmap: &glyph_bitmap,
            width: 1,
            height: 1,
            xmin: 0,
            ymin: 0,
            advance_width: 1,
        }];
        let font = make_bitmap_font(&glyphs);
        let renderer = Glyphr::new();

        let mut target = Callbacks::new(4, 4, |_x, _y, _color| true);
        let result = render_glyph(0, 0, 'Z', font, &renderer, 1.0, &mut target);

        assert!(matches!(result, Err(GlyphrError::InvalidGlyph('Z'))));
    }
}
