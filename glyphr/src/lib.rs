//! # Glyphr
//!
//! This library focus is not to be the fastest, but one of the most beautiful in the embedded world.

#![no_std]

mod api;
mod renderer;
mod rle;
mod utils;

pub use api::{Callbacks, Glyphr, GlyphrError, RenderConfig, SdfConfig, TextAlign};
pub use glyphr_types::{AlignH, AlignV, BitmapFormat, Font, Glyph};

pub use glyphr_macros::generate_font;
#[cfg(feature = "toml")]
pub use glyphr_macros::generate_fonts_from_toml;
