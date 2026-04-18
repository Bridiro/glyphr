#![no_std]

/// Defines how glyph bitmaps are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitmapFormat {
    SDF,
    Bitmap,
}

/// Single glyph data.
pub struct Glyph<'a> {
    pub character: char,
    pub bitmap: &'a [u8],
    pub width: u16,
    pub height: u16,
    pub xmin: i16,
    pub ymin: i16,
    pub advance_width: i16,
}

/// Font metadata and glyph table.
#[derive(Clone, Copy)]
pub struct Font<'a> {
    pub glyphs: &'a [Glyph<'a>],
    pub size: u16,
    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
    pub format: BitmapFormat,
}

impl<'a> Font<'a> {
    /// Returns the glyph if present.
    pub fn find_glyph(&self, ch: char) -> Option<&Glyph<'a>> {
        self.glyphs
            .binary_search_by_key(&ch, |g| g.character)
            .ok()
            .map(|idx| &self.glyphs[idx])
    }
}

/// Horizontal alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignH {
    Left,
    Center,
    Right,
}

/// Vertical alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignV {
    Top,
    Center,
    Baseline,
}
