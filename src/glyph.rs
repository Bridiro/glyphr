//! # glyph.rs
//!
//! Definitions for glyph bounding boxes and metrics used in SDF rendering.

/// Bounding box of the glyph outline, in float coordinates.
/// Generally used only for some precision cases.
#[allow(unused)]
#[repr(C)]
pub struct OutlineBounds {
    pub xmin: f32,
    pub ymin: f32,
    pub width: f32,
    pub height: f32
}

/// Glyph metrics, including bounding box and advance width.
/// Used internally to calculate how and where to place glyphs.
/// 
/// ### Explaination:
/// Each glyph has its own size (height and width), and it's placed differently based on how it's
/// made. Take for example `g` and `t`. With this 2 you can clearly see that `g` goes lower than `t`, and the opposite applies as well.
/// `advance_width` is used to calculate how far away the next glyph should be placed for optimal
/// font reading.
#[allow(unused)]
#[repr(C)]
pub struct Metrics {
    pub xmin: i32,
    pub ymin: i32,
    pub width: i32,
    pub height: i32,
    pub advance_width: f32,
    pub bounds: OutlineBounds,
}

/// One glyph entry in the font, including raw data, resolution, and metrics.
/// This is all used to link together every needed information to correcly display fonts.
#[repr(C)]
pub struct GlyphEntry {
    pub glyph: &'static [u8],
    pub px: u32,
    pub metrics: Metrics,
}
