//! # Glyphr
//! 
//! This library focus is not to be the fastest, but one of the most beautiful in the embedded world.

#![no_std]

pub mod font;
pub mod renderer;
pub mod sdf;
pub mod utils;

pub use font::{AlignH, AlignV, BitmapFormat, Font, Glyph};
pub use glyphr_macros::generate_font;
pub use renderer::{
    BufferTarget, Glyphr, GlyphrError, RenderConfig, RenderTarget, SdfConfig, TextAlign,
};
