//! # font.rs
//!
//! Contains structures used to describe generated fonts

use crate::GlyphrError;

/// Defines how the glyphs are stored in the bitmaps
#[derive(Clone, Copy)]
pub enum BitmapFormat {
    SDF,
    Bitmap,
}

/// Contains informations that are bound to the single glyph
pub struct Glyph<'a> {
    pub character: char,
    pub bitmap: &'a [u8],
    pub width: i32,
    pub height: i32,
    pub xmin: i32,
    pub ymin: i32,
    pub advance_width: i32,
}

/// Contains informations that are useful for every glyph
#[derive(Clone, Copy)]
pub struct Font<'a> {
    pub glyphs: &'a [Glyph<'a>],
    pub size: i32,
    pub ascent: i32,
    pub descent: i32,
    pub format: BitmapFormat,
}

impl<'a> Font<'a> {
    /// Returns a Result, Glyph if it's Ok, Err if the glyph is not found
    pub fn find_glyph(&self, ch: char) -> Result<&Glyph<'a>, GlyphrError> {
        self.glyphs
            .binary_search_by_key(&ch, |g| g.character)
            .map(|idx| &self.glyphs[idx])
            .map_err(|_| GlyphrError::InvalidGlyph(ch))
    }
}

/// Used to describe alignment on X axis
#[derive(Clone, Copy)]
pub enum AlignH {
    Left,
    Center,
    Right,
}

/// Used to describe alignment on Y axis
#[derive(Clone, Copy)]
pub enum AlignV {
    Top,
    Center,
    Baseline,
}
