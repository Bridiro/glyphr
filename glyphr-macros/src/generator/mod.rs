pub mod font;
pub mod font_geometry;
pub mod line;
pub mod sdf_generation;
pub mod vec2;

use crate::config::BitmapFormat;

/// Contains the info of the font to write out (one per glyph)
pub struct GlyphEntry {
    pub name: String,
    pub xmin: i32,
    pub ymin: i32,
    pub width: i32,
    pub height: i32,
    pub advance_width: i32,
}

/// Based on the input, generates a font and return Vec<(bitmaps, entries)> paired
pub fn generate_font(loaded_font: &crate::config::FontLoaded) -> Vec<(Vec<u8>, GlyphEntry)> {
    let mut entries: Vec<(Vec<u8>, GlyphEntry)> = vec![];

    let (spread, padding) = match loaded_font.format {
        BitmapFormat::Bitmap { spread, padding } => (spread, padding),
        BitmapFormat::SDF { spread, padding } => (spread, padding),
    };

    for c in &loaded_font.char_range {
        if let Some((metrics, glyph_sdf)) =
            loaded_font
                .font
                .sdf_generate(loaded_font.px as f32, padding, spread, *c)
        {
            let mut bitmap_sdf = sdf_generation::sdf_to_bitmap(&glyph_sdf);
            let bitmap = match loaded_font.format {
                BitmapFormat::Bitmap {
                    spread: _,
                    padding: _,
                } => {
                    bitmap_sdf = sdf_generation::sdf_bitmap_to_fixed_bitmap(
                        &bitmap_sdf,
                        metrics.width,
                        metrics.height,
                        |val| val > 128,
                    );
                    bitmap_sdf
                }
                BitmapFormat::SDF {
                    spread: _,
                    padding: _,
                } => {
                    rle_encode(bitmap_sdf)
                }
            };
            entries.push((bitmap, GlyphEntry {
                name: format!("GLYPH_{}", *c as u32),
                xmin: metrics.xmin,
                ymin: metrics.ymin,
                width: metrics.width,
                height: metrics.height,
                advance_width: metrics.advance_width,
            }));
        } else {
            let metrics = loaded_font.font.metrics(*c, loaded_font.px as f32);
            if c.is_whitespace() || metrics.map_or(false, |m| m.advance_width > 0) {
                eprintln!(
                    "Info: Glyph '{}' is empty or not renderable, but has metrics. Inserting dummy entry.",
                    c
                );
                let met = metrics.unwrap_or_default();
                entries.push((Vec::new(), GlyphEntry {
                    name: format!("GLYPH_{}", *c as u32),
                    xmin: met.xmin,
                    ymin: met.ymin,
                    width: met.width,
                    height: met.height,
                    advance_width: met.advance_width,
                }));
                continue;
            }

            panic!(
                "font is not complete! U+{:04X} '{}' is not present",
                *c as u32, c
            );
        }
    }

    entries
}

/// Encodes a u8 vector with Run-Lenght-Encoding (RLE)
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
