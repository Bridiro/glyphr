mod config;
mod generator;

use minijinja::{Environment, context};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::env;

use crate::generator::{font::Metrics, sdf_generation};

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var("DOCS_RS").is_ok() {
        return Ok(());
    };

    let loaded_fonts = config::get_config();

    let mut env = Environment::new();
    env.add_template("fonts", include_str!("../templates/fonts.rs.j2"))
        .unwrap();

    let mut all_fonts: Vec<(&str, Vec<Vec<u8>>, Vec<GlyphEntry>)> = Vec::new();
    for loaded_font in &loaded_fonts {
        let (bitmaps, entries) = generate_font(loaded_font);
        all_fonts.push((&loaded_font.name, bitmaps, entries));
    }

    let mut fonts_meta = vec![];

    for (font_name, bitmaps, entries) in &all_fonts {
        let mut glyphs = vec![];

        for (entry, bitmap) in entries.iter().zip(bitmaps.iter()) {
            glyphs.push(context! {
                codepoint => entry.name.clone(),
                bitmap_len => bitmap.len(),
                bitmap => bitmap.clone(),
                px => entry.px,
                metrics => context! {
                    xmin => entry.metrics.xmin,
                    ymin => entry.metrics.ymin,
                    width => entry.metrics.width,
                    height => entry.metrics.height,
                    advance_width => entry.metrics.advance_width,
                    bounds => context! {
                        xmin => entry.metrics.bounds.xmin,
                        ymin => entry.metrics.bounds.ymin,
                        width => entry.metrics.bounds.width,
                        height => entry.metrics.bounds.height,
                    }
                }
            });
        }

        fonts_meta.push(context! {
            name => *font_name,
            glyphs => glyphs,
        });
    }

    // Now render fonts.rs with everything
    let rendered = env
        .get_template("fonts")
        .unwrap()
        .render(context! {
            fonts => fonts_meta,
        })
        .unwrap();
    fs::write(Path::new(&env::var("OUT_DIR")?).join("fonts.rs"), rendered).unwrap();

    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct GlyphEntry {
    pub name: String,
    pub px: u32,
    pub metrics: Metrics,
}

fn generate_font(
    loaded_font: &crate::config::FontLoaded,
) -> (Vec<Vec<u8>>, Vec<GlyphEntry>) {
    let mut bitmaps: Vec<Vec<u8>> = vec![];
    let mut entries: Vec<GlyphEntry> = vec![];

    for c in loaded_font.char_range[0]..=loaded_font.char_range[1] {
        if let Some((metrics, glyph_sdf)) = loaded_font.font.sdf_generate(
            loaded_font.px,
            loaded_font.padding,
            loaded_font.spread,
            c as char,
        ) {
            let bitmap_sdf = sdf_generation::sdf_to_bitmap(&glyph_sdf);
            entries.push(GlyphEntry {
                name: format!("GLYPH_{}", c as u8),
                px: loaded_font.px as u32,
                metrics,
            });
            bitmaps.push(rle_encode(bitmap_sdf.buffer));
        } else {
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
