//! # glyph
//!
//! Definitions for glyph bounding boxes and metrics used in SDF rendering.

/// Bounding box of the glyph outline, in float coordinates.
#[allow(unused)]
#[repr(C)]
pub struct OutlineBounds {
    pub xmin: f32,
    pub ymin: f32,
    pub width: f32,
    pub height: f32
}

/// Glyph metrics, including bounding box and advance width.
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
#[repr(C)]
pub struct GlyphEntry {
    pub glyph: &'static [u8],
    pub px: u32,
    pub metrics: Metrics,
}
