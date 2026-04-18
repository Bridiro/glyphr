use std::{collections::HashMap, ops::Deref};
use ttf_parser::{Face, FaceParsingError};

use crate::generator::{
    font_geometry::{FontGeometry, OutlineBounds},
    line::Line,
    sdf_generation::{SdfRaster, sdf_generate},
};

#[derive(Copy, Clone, Default)]
pub struct FontSettings {
    pub collection_index: u32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct LineMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub new_line_size: f32,
}

impl LineMetrics {
    fn new(ascent: i16, descent: i16, line_gap: i16) -> LineMetrics {
        let (ascent, descent, line_gap) = (ascent as i32, descent as i32, line_gap as i32);
        LineMetrics {
            ascent: ascent as f32,
            descent: descent as f32,
            line_gap: line_gap as f32,
            new_line_size: (ascent - descent + line_gap) as f32,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Metrics {
    pub xmin: i16,
    pub ymin: i16,
    pub width: u16,
    pub height: u16,
    pub advance_width: i16,
}

#[derive(Default)]
pub(crate) struct Glyph {
    pub bounds: OutlineBounds,
    pub advance_width: f32,
    pub lines: Vec<Line>,
}

pub struct Font {
    glyphs: HashMap<char, Glyph>,
    horizontal_line_metrics: LineMetrics,
    units_per_em: f32,
}

impl Font {
    pub fn from_bytes<D: Deref<Target = [u8]>>(
        data: D,
        settings: FontSettings,
    ) -> Result<Self, FaceParsingError> {
        let face = Face::parse(&data, settings.collection_index)?;
        let units_per_em = face.units_per_em() as f32;

        let glyph_count = face.number_of_glyphs();
        let mut glyph_id_mapping = HashMap::with_capacity(glyph_count as usize);
        if let Some(subtable) = face.tables().cmap {
            for subtable in subtable.subtables {
                subtable.codepoints(|codepoint| {
                    if let Some(mapping) = subtable.glyph_index(codepoint) {
                        glyph_id_mapping.insert(codepoint, mapping);
                    }
                })
            }
        }

        let mut glyphs = HashMap::with_capacity(glyph_id_mapping.len());
        for (codepoint, glyph_id) in glyph_id_mapping {
            let char = match char::from_u32(codepoint) {
                Some(c) => c,
                None => continue,
            };

            let mut glyph = Glyph::default();

            let mut geometry = FontGeometry::new();
            face.outline_glyph(glyph_id, &mut geometry);
            geometry.finalize();

            glyph.lines = geometry.lines;
            glyph.advance_width = face.glyph_hor_advance(glyph_id).unwrap_or(0) as f32;
            glyph.bounds = geometry.bounds;

            glyphs.insert(char, glyph);
        }

        let horizontal_line_metrics =
            LineMetrics::new(face.ascender(), face.descender(), face.line_gap());

        let font = Font {
            glyphs,
            units_per_em,
            horizontal_line_metrics,
        };

        Ok(font)
    }

    pub fn metrics(&self, c: char, px: f32) -> Option<Metrics> {
        let scale = self.scale_factor(px);

        let glyph = self.glyphs.get(&c)?;

        let bounds = glyph.bounds.scale(scale);
        let metrics = Metrics {
            xmin: (bounds.xmin as i32)
                .try_into()
                .expect("glyph xmin does not fit into i16"),
            ymin: (bounds.ymin as i32)
                .try_into()
                .expect("glyph ymin does not fit into i16"),
            width: (bounds.width as i32)
                .try_into()
                .expect("glyph width does not fit into u16"),
            height: (bounds.height as i32)
                .try_into()
                .expect("glyph height does not fit into u16"),
            advance_width: ((glyph.advance_width * scale) as i32)
                .try_into()
                .expect("glyph advance_width does not fit into i16"),
        };

        Some(metrics)
    }

    pub fn sdf_generate(
        &self,
        px: f32,
        padding: i32,
        spread: f32,
        c: char,
    ) -> Option<(Metrics, SdfRaster)> {
        if px < 1.0 {
            panic!("Sdf render size cannot be smaller than 1.0 (got {px:?})");
        }

        let glyph = match self.glyphs.get(&c) {
            Some(g) => g,
            None => {
                return None;
            }
        };

        let metrics = self.metrics(c, px).unwrap(); // Cannot return `None` if glyph is some

        let sdf = sdf_generate(
            u32::from(metrics.width),
            u32::from(metrics.height),
            padding,
            spread,
            &glyph.lines,
        );

        Some((metrics, sdf))
    }

    fn scale_factor(&self, px: f32) -> f32 {
        px / self.units_per_em
    }

    pub fn get_ascent(&self, px: f32) -> i16 {
        ((self.horizontal_line_metrics.ascent * self.scale_factor(px)) as i32)
            .try_into()
            .expect("font ascent does not fit into i16")
    }

    pub fn get_descent(&self, px: f32) -> i16 {
        ((self.horizontal_line_metrics.descent * self.scale_factor(px)) as i32)
            .try_into()
            .expect("font descent does not fit into i16")
    }

    pub fn get_line_gap(&self, px: f32) -> i16 {
        ((self.horizontal_line_metrics.line_gap * self.scale_factor(px)) as i32)
            .try_into()
            .expect("font line_gap does not fit into i16")
    }
}
