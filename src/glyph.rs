#[allow(unused)]
#[repr(C)]
pub struct OutlineBounds {
    pub xmin: f32,
    pub ymin: f32,
    pub width: f32,
    pub height: f32
}

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

#[repr(C)]
pub struct GlyphEntry {
    pub glyph: &'static [u8],
    pub px: u32,
    pub metrics: Metrics,
}
