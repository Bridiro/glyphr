use serde::Deserialize;
use std::fmt;

use crate::generator::font::Font;

/// Trait used internally to define which struct can define a font.
pub trait ToFontLoaded {
    fn to_font_loaded(&self) -> Vec<FontLoaded>;
}

/// Defines with which method to generate the font bitmap.
#[derive(PartialEq, Deserialize, Copy, Clone)]
pub enum BitmapFormat {
    SDF { spread: f32, padding: i32 },
    Bitmap { spread: f32, padding: i32 },
}

impl fmt::Display for BitmapFormat {
    /// used on output to print out the format in the format used by Glyphr internally
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BitmapFormat::Bitmap {
                spread: _,
                padding: _,
            } => write!(f, "BitmapFormat::Bitmap"),
            BitmapFormat::SDF {
                spread: _,
                padding: _,
            } => {
                write!(f, "BitmapFormat::SDF")
            }
        }
    }
}

/// Last stage of font informations before generation.
pub struct FontLoaded {
    pub name: String,
    pub font: Font,
    pub px: i32,
    pub char_range: Vec<char>,
    pub format: BitmapFormat,
}

/// The input is a regex-like string, and the output is the "regex" extended as an array
pub fn parse_char_set(pattern: &str) -> Vec<char> {
    let mut chars = Vec::new();
    let mut chars_iter = pattern.chars().peekable();

    let mut last = None;

    while let Some(c) = chars_iter.next() {
        match c {
            '-' if last.is_some() && chars_iter.peek().is_some() => {
                let start = last.unwrap() as u32;
                let end = *chars_iter.peek().unwrap() as u32;

                if start < end {
                    chars.extend((start + 1..=end).filter_map(std::char::from_u32));
                }

                last = Some(chars_iter.next().unwrap());
                chars.push(last.unwrap());
            }
            _ => {
                chars.push(c);
                last = Some(c);
            }
        }
    }

    chars.sort_unstable();
    chars.dedup();
    chars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_char_set() {
        let input = "A-Da-d0-3";
        let output = parse_char_set(input);
        let predicted_output = ['0', '1', '2', '3', 'A', 'B', 'C', 'D', 'a', 'b', 'c', 'd'];
        assert_eq!(output, predicted_output);
    }
}
