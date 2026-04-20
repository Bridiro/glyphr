#![no_std]

/// Defines how glyph bitmaps are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitmapFormat {
    /// SDF (signed distance field) format, where each byte represents the signed distance to the nearest edge.
    SDF,
    /// Bitmap format, where each byte contains 8 pixels of 1-bit coverage data, packed MSB to LSB. The bitmap is tightly packed with no padding.
    Bitmap,
}

/// Single glyph data.
pub struct Glyph<'a> {
    /// The character this glyph represents.
    pub character: char,
    /// The bitmap data for this glyph, in the format specified by the parent font. The bitmap is tightly packed with no padding.
    pub bitmap: &'a [u8],
    /// The width of the bitmap in pixels.
    pub width: u16,
    /// The height of the bitmap in pixels.
    pub height: u16,
    /// The horizontal offset from the pen position to the left edge of the bitmap.
    pub xmin: i16,
    /// The vertical offset from the pen position to the top edge of the bitmap. Note that in typical font coordinate systems, y increases upwards, so this value is often negative.
    pub ymin: i16,
    /// The horizontal distance to advance the pen position after rendering this glyph.
    pub advance_width: i16,
}

/// Font metadata and glyph table.
#[derive(Clone, Copy)]
pub struct Font<'a> {
    /// The list of glyphs in this font, sorted by the `character` field for efficient lookup.
    pub glyphs: &'a [Glyph<'a>],
    /// The font size in pixels (the nominal height of the font).
    pub size: u16,
    /// The ascent of the font (the distance from the baseline to the highest point of any glyph).
    pub ascent: i16,
    /// The descent of the font (the distance from the baseline to the lowest point of any glyph, typically negative).
    pub descent: i16,
    /// The line gap (the additional vertical space to add between lines of text, typically positive).
    pub line_gap: i16,
    /// The format of the bitmap data for the glyphs in this font. This applies to all glyphs in the font and determines how to interpret the `bitmap` data in each `Glyph`.
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
    /// Align text to the left of the pen position.
    Left,
    /// Align text centered on the pen position.
    Center,
    /// Align text to the right of the pen position.
    Right,
}

/// Vertical alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignV {
    /// Align text with the top of the tallest glyph at the pen position.
    Top,
    /// Align text with the vertical center of the tallest glyph at the pen position.
    Center,
    /// Align text with the baseline of the glyphs.
    Baseline,
}
