use easy_signed_distance_field as sdf;
use serde_json;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::env;
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
    let json_fonts = fs::read_to_string(Path::new(&manifest_dir).join("fonts/fonts.json"))?;
    let loaded_fonts: Vec<FontDescriptor> = serde_json::from_str(&json_fonts)?;

    for loaded_font in loaded_fonts {
        let font_file = fs::read(Path::new(&manifest_dir).join(loaded_font.path)).expect("Can't read file");
        let font = sdf::Font::from_bytes(font_file.as_slice(), Default::default()).expect("Failed to parse font file");

        let px = loaded_font.px;
        let padding = loaded_font.padding;
        let spread = loaded_font.spread;

        for c in loaded_font.char_range[0]..=loaded_font.char_range[1] {
            if let Some((metrics, glyph_sdf)) = font.sdf_generate(px, padding, spread, c as char) {
            }
        }
    }
    Ok(())
}
