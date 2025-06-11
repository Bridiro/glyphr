use crate::GlyphrError;

pub struct Glyph<'a> {
    pub character: char,
    pub bitmap: &'a [u8],
    pub width: i32,
    pub height: i32,
    pub xmin: i32,
    pub ymin: i32,
    pub advance_width: i32,
}

#[derive(Clone, Copy)]
pub enum BitmapFormat {
    SDF,
    Bitmap,
}

#[derive(Clone, Copy)]
pub struct Font<'a> {
    pub glyphs: &'a [Glyph<'a>],
    pub size: i32,
    pub ascent: i32,
    pub descent: i32,
    pub format: BitmapFormat,
}

impl<'a> Font<'a> {
    pub fn find_glyph(&self, ch: char) -> Result<&Glyph<'a>, GlyphrError> {
        self.glyphs
            .binary_search_by_key(&ch, |g| g.character)
            .map(|idx| &self.glyphs[idx])
            .map_err(|_| GlyphrError::InvalidGlyph(ch))
    }
}

#[derive(Clone, Copy)]
pub enum AlignH {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
pub enum AlignV {
    Top,
    Center,
    Baseline,
}
