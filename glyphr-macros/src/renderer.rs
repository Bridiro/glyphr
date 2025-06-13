use minijinja::{Environment, context};

use crate::config::ToFontLoaded;
use crate::generator::generate_font;

pub fn render<T: ToFontLoaded>(font_config: T) -> String {
    let loaded_fonts = font_config.to_font_loaded();

    let mut env = Environment::new();
    env.add_template("fonts", include_str!("../templates/fonts.rs.j2"))
        .unwrap();

    let mut output = String::new();
    for loaded_font in &loaded_fonts {
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
        output.push_str(&env.get_template("fonts")
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
            .unwrap());
    }

    output
}
