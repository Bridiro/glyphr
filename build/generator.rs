pub mod font;
pub mod font_geometry;
pub mod line;
pub mod sdf_generation;
pub mod vec2;

use font::Metrics;

pub struct GlyphEntry {
    pub name: String,
    pub px: u32,
    pub metrics: Metrics,
}

pub fn generate_font(
    loaded_font: &crate::config::FontLoaded,
) -> (Vec<Vec<u8>>, Vec<GlyphEntry>) {
    let mut bitmaps: Vec<Vec<u8>> = vec![];
    let mut entries: Vec<GlyphEntry> = vec![];

    for c in &loaded_font.char_range {
        if let Some((metrics, glyph_sdf)) = loaded_font.font.sdf_generate(
            loaded_font.px,
            loaded_font.padding,
            loaded_font.spread,
            *c as char,
        ) {
            let bitmap_sdf = sdf_generation::sdf_to_bitmap(&glyph_sdf);
            entries.push(GlyphEntry {
                name: format!("GLYPH_{}", *c as u8),
                px: loaded_font.px as u32,
                metrics,
            });
            bitmaps.push(rle_encode(bitmap_sdf.buffer));
        } else {
            // Check if the character is a space or similar invisible character
            let ch = *c as char;
            let metrics = loaded_font.font.metrics(ch, loaded_font.px);
            if ch.is_whitespace() || metrics.map_or(false, |m| m.advance_width > 0.0) {
                eprintln!("Info: Glyph '{}' is empty or not renderable, but has metrics. Inserting dummy entry.", ch);
                entries.push(GlyphEntry {
                    name: format!("GLYPH_{}", *c as u8),
                    px: loaded_font.px as u32,
                    metrics: metrics.unwrap_or_default(),
                });
                bitmaps.push(Vec::new());
                continue;
            }

            panic!("font is not complete!");
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

