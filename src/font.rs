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
pub struct Font<'a> {
    pub glyphs: &'a [Glyph<'a>],
    pub size: i32,
    pub ascent: i32,
    pub descent: i32,
}

impl<'a> Font<'a> {
    pub fn find_glyph(&self, ch: char) -> Option<&Glyph<'a>> {
        self.glyphs
            .binary_search_by_key(&ch, |g| g.character)
            .ok()
            .map(|idx| &self.glyphs[idx])
    }
}

pub enum AlignH {
    Left,
    Center,
    Right,
}

pub enum AlignV {
    Top,
    Center,
    Baseline,
}
