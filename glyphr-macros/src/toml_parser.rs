use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::config::{BitmapFormat, FontLoaded, ToFontLoaded, parse_char_set};
use crate::generator::font::Font;

/// Contains all the fonts specified in the `toml`
#[derive(Deserialize)]
pub struct TomlConfig {
    pub font: Vec<TomlFont>,
}

/// Describes one font in the `toml`
#[derive(Deserialize)]
pub struct TomlFont {
    pub name: String,
    pub path: String,
    pub size: i32,
    pub characters: String,
    pub format: BitmapFormat,
}

impl ToFontLoaded for TomlConfig {
    fn to_font_loaded(&self) -> Vec<FontLoaded> {
        let mut fonts = Vec::new();

        for toml_font in &self.font {
            let ttf_file = fs::read(Path::new(&toml_font.path))
                .expect(&format!("can't read ttf file at path: {}", toml_font.path));
            let font = Font::from_bytes(ttf_file.as_slice(), Default::default())
                .expect("failed to parse ttf file");
            fonts.push(FontLoaded {
                name: toml_font.name.to_string(),
                font,
                px: toml_font.size,
                char_range: parse_char_set(&toml_font.characters),
                format: toml_font.format,
            });
        }

        fonts
    }
}

impl TomlConfig {
    /// Used to relativize the paths of ttfs to the relative path of the `toml` file
    pub fn relativize_paths(&mut self, toml_path: &str) {
        let toml_path = Path::new(toml_path);

        let base_dir = toml_path.parent().unwrap_or(Path::new(""));

        for font in &mut self.font {
            let font_path = Path::new(&font.path);

            if font_path.is_relative() {
                let full_path = base_dir.join(font_path);
                font.path = full_path.to_string_lossy().to_string();
            }
        }
    }
}
