use std::fmt;

use crate::generator::font::Font;

#[derive(PartialEq)]
pub enum BitmapFormat {
    SDF { spread: f32, padding: i32 },
    Bitmap { spread: f32, padding: i32 },
}

impl fmt::Display for BitmapFormat {
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

pub struct FontLoaded {
    pub name: String,
    pub font: Font,
    pub px: i32,
    pub char_range: Vec<char>,
    pub format: BitmapFormat,
}

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
