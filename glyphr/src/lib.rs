//! # Glyphr
//!
//! This library focus is not to be the fastest, but one of the most beautiful in the embedded world.

#![no_std]

mod api;
mod font;
mod renderer;
mod utils;

pub use api::{
    BufferTarget, Glyphr, GlyphrError, RenderConfig, RenderTarget, SdfConfig, TextAlign,
};
pub use font::{AlignH, AlignV, BitmapFormat, Font, Glyph};
pub use glyphr_macros::generate_font;

#[cfg(feature = "toml")]
pub use glyphr_macros::generate_fonts_from_toml;
