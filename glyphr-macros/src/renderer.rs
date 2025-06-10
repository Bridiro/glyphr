use minijinja::{Environment, context};
use std::fs;
use std::path::Path;

use crate::generator::font::Font;
use crate::macro_parser::FontConfig;
use crate::{
    config::{FontLoaded, parse_char_set},
    generator::generate_font,
};

pub fn render(font_config: FontConfig) -> String {
    let loaded_font = font_config_to_loaded(font_config);

    let mut env = Environment::new();
    env.add_template("fonts", include_str!("../templates/fonts.rs.j2"))
        .unwrap();

    let mut glyphs = vec![];

    let (bitmaps, entries) = generate_font(&loaded_font);

    for ((entry, bitmap), character) in entries
        .iter()
        .zip(bitmaps.iter())
        .zip(loaded_font.char_range.iter())
    {
        glyphs.push(context! {
            character => character,
            codepoint => entry.name.clone(),
            bitmap_len => bitmap.len(),
            bitmap => bitmap.clone(),
            xmin => entry.metrics.xmin,
            ymin => entry.metrics.ymin,
            width => entry.metrics.width,
            height => entry.metrics.height,
            advance_width => entry.metrics.advance_width,
        });
    }

    // Now render fonts.rs with everything
    env.get_template("fonts")
        .unwrap()
        .render(context! {
            font => context! {
                name => loaded_font.name,
                size => loaded_font.px,
                ascent => loaded_font.font.get_ascent(loaded_font.px as f32),
                descent => loaded_font.font.get_descent(loaded_font.px as f32),
                format => loaded_font.format.to_string(),
                glyphs => glyphs,
            },
        })
        .unwrap()
}

fn font_config_to_loaded(font_config: FontConfig) -> FontLoaded {
    let ttf_file = fs::read(Path::new(&font_config.path)).expect(&format!(
        "can't read ttf file at path: {}",
        font_config.path
    ));
    let font = Font::from_bytes(ttf_file.as_slice(), Default::default())
        .expect("failed to parse ttf file");
    FontLoaded {
        name: font_config.name.to_string(),
        font,
        px: font_config.size,
        char_range: parse_char_set(&font_config.characters),
        format: font_config.format,
    }
}
