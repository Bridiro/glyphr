use minijinja::{Environment, context};

use crate::{config::FontLoaded, generator::generate_font};

pub fn render(loaded_fonts: Vec<FontLoaded>) -> String {
    let mut env = Environment::new();
    env.add_template("fonts", include_str!("../templates/fonts.rs.j2"))
        .unwrap();

    let mut fonts_meta = vec![];

    for loaded_font in &loaded_fonts {
        let mut glyphs = vec![];

        let (bitmaps, entries) = generate_font(loaded_font);

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
            name => loaded_font.name,
            glyphs => glyphs,
        });
    }

    // Now render fonts.rs with everything
    env.get_template("fonts")
        .unwrap()
        .render(context! {
            fonts => fonts_meta,
        })
        .unwrap()
}
