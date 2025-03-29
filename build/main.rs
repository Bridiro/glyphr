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
use std::path::Path;

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
    println!("cargo::rerun-if-changed=fonts/fonts.json");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let fonts_json_path = fs::read_to_string(Path::new(&manifest_dir).join("fonts/fonts.json"))?;
    let loaded_fonts: Vec<FontDescriptor> = serde_json::from_str(&fonts_json_path)?;

    let mut file = fs::File::create(Path::new(&manifest_dir).join("src/fonts.rs"))?;

    for loaded_font in loaded_fonts {
        let font_file =
            fs::read(Path::new(&manifest_dir).join(&loaded_font.path)).expect("Can't read file");
        let font = font::Font::from_bytes(font_file.as_slice(), Default::default())
            .expect("Failed to parse font file");

        generate_and_write_font_to_file(&font, &loaded_font, &mut file)?;
    }
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct GlyphEntry {
    pub metrics: font::Metrics,
    pub sdf_offset: u32,
}

fn generate_and_write_font_to_file(
    font: &font::Font,
    loaded_font: &FontDescriptor,
    file: &mut fs::File,
) -> Result<(), Box<dyn Error>> {
    let mut bitmaps: Vec<u8> = vec![];
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
                metrics,
                sdf_offset: bitmaps.len() as u32,
            });
            bitmaps.extend(rle_encode(bitmap_sdf.buffer));
        }
    }

    file.write_all(
        b"pub struct OutlineBounds {
    pub xmin: f32,
    pub ymin: f32,
    pub width: f32,
    pub height: f32
}\n\n",
    )?;

    file.write_all(
        b"pub struct Metrics {
    pub xmin: i32,
    pub ymin: i32,
    pub width: i32,
    pub height: i32,
    pub advance_width: f32,
    pub bounds: OutlineBounds,
}\n\n",
    )?;

    file.write_all(
        b"pub struct GlyphEntry {
    pub metrics: Metrics,
    pub sdf_offset: u32,
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
        file.write_all(format!("    {:#?},\n", entry).as_bytes())?;
    }
    file.write_all(b"];\n\n")?;

    file.write_all(
        format!(
            "pub static FONT_{}: [u8; {}] = [",
            loaded_font.name.to_uppercase(),
            bitmaps.len()
        )
        .as_bytes(),
    )?;
    for (i, byte) in bitmaps.iter().enumerate() {
        if i % 16 == 0 {
            writeln!(file)?;
        }
        write!(file, "{}, ", byte)?;
    }
    writeln!(file, "\n];")?;

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
