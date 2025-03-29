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
use std::path::Path;
use std::io::prelude::*;

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

fn generate_and_write_font_to_file(font: &font::Font, loaded_font: &FontDescriptor, file: &mut fs::File) -> Result<(), Box<dyn Error>> {
    let mut bitmaps: Vec<u8> = vec![];

    for c in loaded_font.char_range[0]..=loaded_font.char_range[1] {
        if let Some((_metrics, glyph_sdf)) = font.sdf_generate(loaded_font.px, loaded_font.padding, loaded_font.spread, c as char) {
            let bitmap_sdf = sdf_generation::sdf_to_bitmap(&glyph_sdf);
            bitmaps.extend(rle_encode(bitmap_sdf.buffer));
        }
    }

    file.write_all(format!("const FONT_{}: [u8, {}] = [", loaded_font.name, bitmaps.len()).as_bytes())?;
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
