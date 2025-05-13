mod font;
mod font_geometry;
mod line;
mod sdf_generation;
mod vec2;

use minijinja::{Environment, context};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

#[derive(Serialize, Deserialize, Debug)]
struct FontDescriptor {
    name: String,
    path: String,
    px: f32,
    padding: i32,
    spread: f32,
    char_range: Vec<u8>,
}

#[derive(Deserialize)]
struct Config {
    font: Vec<FontDescriptor>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = if let Ok(path) = env::var("FONTS_DIR") {
        PathBuf::from(path)
    } else {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let mut current_dir = out_dir;

        loop {
            if current_dir.join("Cargo.toml").exists() {
                break current_dir.join("fonts");
            }
            if !current_dir.pop() {
                panic!("Failed to locate project root containing Cargo.toml");
            }
        }
    };

    // Verify the configuration file exists
    if !config_path.exists() {
        panic!("Configuration file not found at {}", config_path.display());
    }
    println!(
        "cargo::rerun-if-changed={}",
        config_path.join("fonts.toml").display()
    );

    let fonts_toml_path = fs::read_to_string(Path::new(&config_path).join("fonts.toml"))
        .expect("Could not open or find fonts.toml");
    let loaded_fonts_config: Config =
        toml::from_str(&fonts_toml_path).expect("Error parsing fonts.toml");
    let loaded_fonts = loaded_fonts_config.font;

    let mut env = Environment::new();
    env.add_template("fonts", include_str!("../templates/fonts.rs.j2"))
        .unwrap();

    let mut all_fonts: Vec<(&str, Vec<Vec<u8>>, Vec<GlyphEntry>)> = Vec::new();
    for loaded_font in &loaded_fonts {
        let font_file = fs::read(Path::new(&config_path).join(&loaded_font.path))
            .expect(&format!("Can't read ttf file {}", loaded_font.path));
        let font = font::Font::from_bytes(font_file.as_slice(), Default::default())
            .expect(&format!("Failed to parse font file: {}", loaded_font.path));

        let (bitmaps, entries) = generate_font(&font, loaded_font);
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
let rendered = env.get_template("fonts").unwrap().render(context! {
    fonts => fonts_meta,
}).unwrap();
    fs::write(Path::new(&env::var("OUT_DIR")?).join("fonts.rs"), rendered).unwrap();

    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct GlyphEntry {
    pub name: String,
    pub px: u32,
    pub metrics: font::Metrics,
}

fn generate_font(
    font: &font::Font,
    loaded_font: &FontDescriptor,
) -> (Vec<Vec<u8>>, Vec<GlyphEntry>) {
    let mut bitmaps: Vec<Vec<u8>> = vec![];
    let mut entries: Vec<GlyphEntry> = vec![];

    for c in loaded_font.char_range[0]..=loaded_font.char_range[1] {
        if let Some((metrics, glyph_sdf)) = font.sdf_generate(
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
