use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::generator::font::Font;

pub struct FontLoaded {
    pub name: String,
    pub font: Font,
    pub px: f32,
    pub padding: i32,
    pub spread: f32,
    pub char_range: Vec<u8>,
}

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

pub fn path_to() -> PathBuf {
    if let Ok(path) = env::var("GLYPHR_CONFIG") {
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
    }
}

pub fn get_config() -> Vec<FontLoaded> {
    let config_path = path_to();

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

    let mut fonts: Vec<FontLoaded> = vec![];

    for loaded_font in loaded_fonts_config.font {
        let font_file = fs::read(Path::new(&config_path).join(&loaded_font.path))
            .expect(&format!("Can't read ttf file {}", loaded_font.path));
        let font = Font::from_bytes(font_file.as_slice(), Default::default())
            .expect(&format!("Failed to parse font file: {}", loaded_font.path));
        fonts.push(FontLoaded {
            name: loaded_font.name,
            font,
            px: loaded_font.px,
            padding: loaded_font.padding,
            spread: loaded_font.spread,
            char_range: loaded_font.char_range,
        })
    }

    fonts
}
