use minijinja::{Environment, context};
use std::env;
use std::fs;
use std::path::Path;

use crate::{
    config::FontLoaded,
    generator::{GlyphEntry, generate_font},
};

pub fn render(loaded_fonts: Vec<FontLoaded>) {
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
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("fonts.rs"),
        rendered,
    )
    .unwrap();
}
