use easy_signed_distance_field as sdf;
use serde_json;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct FontDescriptor {
    name: String,
    path: String,
    size: u16,
    char_range: Vec<u16>,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=fonts/fonts.json");
    let json_fonts = fs::read_to_string("fonts/fonts.json")?;
    let loaded_fonts: Vec<FontDescriptor> = serde_json::from_str(&json_fonts)?;

    for loaded_font in loaded_fonts {
        let font_file = fs::read(loaded_font.path).expect("Can't read file");
        let font = sdf::Font::from_bytes(font_file.as_slice(), Default::default()).expect("Failed to parse font file");

        let px = loaded_font.size;
        let padding = 1;
        let spread = 0.6;
    }
    Ok(())
}
