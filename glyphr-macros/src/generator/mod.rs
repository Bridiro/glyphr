pub mod font;
pub mod font_geometry;
pub mod line;
pub mod sdf_generation;
pub mod vec2;

use crate::config::BitmapFormat;
use font::Metrics;

pub struct GlyphEntry {
    pub name: String,
    pub metrics: Metrics,
}

pub fn generate_font(loaded_font: &crate::config::FontLoaded) -> (Vec<Vec<u8>>, Vec<GlyphEntry>) {
    let mut bitmaps: Vec<Vec<u8>> = vec![];
    let mut entries: Vec<GlyphEntry> = vec![];

    let (spread, padding) = match loaded_font.format {
        BitmapFormat::Bitmap => (10.0, 0),
        BitmapFormat::SDF { spread, padding } => (spread, padding),
    };

    for c in &loaded_font.char_range {
        if let Some((metrics, glyph_sdf)) =
            loaded_font
                .font
                .sdf_generate(loaded_font.px as f32, padding, spread, *c)
        {
            let mut bitmap_sdf = sdf_generation::sdf_to_bitmap(&glyph_sdf);
            if loaded_font.format == BitmapFormat::Bitmap {
                bitmap_sdf = sdf_generation::sdf_bitmap_to_fixed_bitmap(
                    &bitmap_sdf,
                    metrics.width,
                    metrics.height,
                    |val| val > 128,
                );
                bitmaps.push(bitmap_sdf);
            } else {
                bitmaps.push(rle_encode(bitmap_sdf));
            }
            entries.push(GlyphEntry {
                name: format!("GLYPH_{}", *c as u32),
                metrics,
            });
        } else {
            let metrics = loaded_font.font.metrics(*c, loaded_font.px as f32);
            if c.is_whitespace() || metrics.map_or(false, |m| m.advance_width > 0) {
                eprintln!(
                    "Info: Glyph '{}' is empty or not renderable, but has metrics. Inserting dummy entry.",
                    c
                );
                entries.push(GlyphEntry {
                    name: format!("GLYPH_{}", *c as u32),
                    metrics: metrics.unwrap_or_default(),
                });
                bitmaps.push(Vec::new());
                continue;
            }

            panic!(
                "font is not complete! U+{:04X} '{}' is not present",
                *c as u32, c
            );
        }
    }

    (bitmaps, entries)
}

fn rle_encode(data: Vec<u8>) -> Vec<u8> {
    let mut encoded = Vec::new();
    let mut iter = data.iter().peekable();

    while let Some(&value) = iter.next() {
        let mut count = 1;
        while let Some(&&next) = iter.peek() {
            if next == value && count < u8::MAX {
                iter.next();
                count += 1;
            } else {
                break;
            }
        }
        encoded.push(count);
        encoded.push(value);
    }

    encoded
}
