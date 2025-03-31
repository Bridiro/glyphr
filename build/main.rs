mod font;
mod font_geometry;
mod line;
mod sdf_generation;
mod vec2;

use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct FontDescriptor {
    name: String,
    path: String,
    px: f32,
    padding: i32,
    spread: f32,
    char_range: Vec<u8>,
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
    println!("cargo::rerun-if-changed={}", config_path.join("fonts.json").display());

    let fonts_json_path = fs::read_to_string(Path::new(&config_path).join("fonts.json"))?;
    let loaded_fonts: Vec<FontDescriptor> = serde_json::from_str(&fonts_json_path)?;

    let mut file = fs::File::create(Path::new(&env::var("OUT_DIR")?).join("generated.rs"))?;

    for loaded_font in loaded_fonts {
        let font_file =
            fs::read(Path::new(&config_path).join(&loaded_font.path)).expect("Can't read file");
        let font = font::Font::from_bytes(font_file.as_slice(), Default::default())
            .expect("Failed to parse font file");

        generate_and_write_font_to_file(&font, &loaded_font, &mut file)?;
    }
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct GlyphEntry {
    pub name: String,
    pub metrics: font::Metrics,
}

fn generate_and_write_font_to_file(
    font: &font::Font,
    loaded_font: &FontDescriptor,
    file: &mut fs::File,
) -> Result<(), Box<dyn Error>> {
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
                name: format!("GLYPH_{}_{}", loaded_font.name.to_uppercase(), c as u8),
                metrics,
            });
            bitmaps.push(rle_encode(bitmap_sdf.buffer));
        }
    }

    file.write_all(
        b"#[allow(unused)]
#[repr(C)]
pub struct OutlineBounds {
    pub xmin: f32,
    pub ymin: f32,
    pub width: f32,
    pub height: f32
}\n\n",
    )?;

    file.write_all(
        b"#[allow(unused)]
#[repr(C)]
pub struct Metrics {
    pub xmin: i32,
    pub ymin: i32,
    pub width: i32,
    pub height: i32,
    pub advance_width: f32,
    pub bounds: OutlineBounds,
}\n\n",
    )?;

    file.write_all(
        b"#[repr(C)]
pub struct GlyphEntry {
    pub glyph: &'static [u8],
    pub metrics: Metrics,
}\n\n",
    )?;

    file.write_all(
        format!(
            "pub static FONT_{}_ENTRIES: [GlyphEntry; {}] = [\n",
            loaded_font.name.to_uppercase(),
            entries.len()
        )
        .as_bytes(),
    )?;
    for entry in entries {
        file.write_all(
            format!(
                "GlyphEntry {{
    glyph: &{},
    metrics: {:#?},\n}},\n",
                entry.name, entry.metrics
            )
            .as_bytes(),
        )?;
    }
    file.write_all(b"];\n\n")?;

    for (i, bitmap) in bitmaps.iter().enumerate() {
        file.write_all(
            format!(
                "pub static GLYPH_{}_{}: [u8; {}] = [",
                loaded_font.name.to_uppercase(),
                i as u8 + loaded_font.char_range[0],
                bitmap.len()
            )
            .as_bytes(),
        )?;
        for (j, byte) in bitmap.iter().enumerate() {
            if j % 16 == 0 {
                writeln!(file)?;
            }
            write!(file, "{}, ", byte)?;
        }
        writeln!(file, "\n];\n\n")?;
    }

    Ok(())
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
