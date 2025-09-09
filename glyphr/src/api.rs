//! # rendered.rs
//!
//! This module describes the public API to this library.
//! Everything is done via the `Glyphr` struct.

use crate::font::{AlignH, AlignV, BitmapFormat, Font};
use crate::renderer;

/// Trait used to make a target writable by Glyphr.
pub trait RenderTarget {
    /// x and y are coordinates, while color contains an ARGB8888 encoded value. You should handle
    /// alpha blending on your own.
    fn write_pixel(&mut self, x: u32, y: u32, color: u32) -> bool;

    /// This function return a touple of (width, height) of the target.
    fn dimensions(&self) -> (u32, u32);
}

/// Built-in implementation for u32 slice buffers.
pub struct BufferTarget<'a> {
    pub buffer: &'a mut [u32],
    pub width: u32,
    pub height: u32,
}

impl<'a> BufferTarget<'a> {
    pub fn new(buffer: &'a mut [u32], width: u32, height: u32) -> Self {
        assert_eq!(
            buffer.len(),
            (width * height) as usize,
            "Buffer size doesn't match dimensions"
        );
        Self {
            buffer,
            width,
            height,
        }
    }
}

impl<'a> RenderTarget for BufferTarget<'a> {
    fn write_pixel(&mut self, x: u32, y: u32, color: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        let index = (y * self.width + x) as usize;

        if index < self.buffer.len() {
            let bg = self.buffer[index];
            let alpha = (color >> 24) & 0xff;
            if alpha == 0xff {
                self.buffer[index] = color;
            } else {
                let alpha_f = alpha as f32 / 255.0;

                let fg_r = (color >> 16) & 0xFF;
                let fg_g = (color >> 8) & 0xFF;
                let fg_b = color & 0xFF;

                let bg_r = (bg >> 16) & 0xFF;
                let bg_g = (bg >> 8) & 0xFF;
                let bg_b = bg & 0xFF;

                let blended_r = ((fg_r as f32 * alpha_f) + (bg_r as f32 * (1.0 - alpha_f))) as u8;
                let blended_g = ((fg_g as f32 * alpha_f) + (bg_g as f32 * (1.0 - alpha_f))) as u8;
                let blended_b = ((fg_b as f32 * alpha_f) + (bg_b as f32 * (1.0 - alpha_f))) as u8;

                let blended = (255 << 24)
                    | ((blended_r as u32) << 17)
                    | ((blended_g as u32) << 8)
                    | (blended_b as u32);
                self.buffer[index] = blended;
            }

            true
        } else {
            false
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// Configuration for text rendering.
#[derive(Clone, Copy)]
pub struct RenderConfig {
    /// Color to render the text.
    pub color: u32,
    /// SDF-specific configuration (ignored for bitmap fonts).
    pub sdf: SdfConfig,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            color: 0xffffff,
            sdf: SdfConfig::default(),
        }
    }
}

/// Configuration for SDF rendering (only used with SDF fonts).
#[derive(Clone, Copy)]
pub struct SdfConfig {
    /// Font size in pixels (only affects SDF fonts).
    pub size: u32,
    /// Mid-value for SDF (usually 0.5).
    pub mid_value: f32,
    /// Smoothing factor for anti-aliasing.
    pub smoothing: f32,
}

impl Default for SdfConfig {
    fn default() -> Self {
        Self {
            size: 16,
            mid_value: 0.5,
            smoothing: 0.1,
        }
    }
}

/// Text alignment options.
#[derive(Clone, Copy)]
pub struct TextAlign {
    pub horizontal: AlignH,
    pub vertical: AlignV,
}

impl Default for TextAlign {
    fn default() -> Self {
        Self {
            horizontal: AlignH::Left,
            vertical: AlignV::Top,
        }
    }
}

/// Main renderer struct. With this you can render code.
pub struct Glyphr {
    render_config: RenderConfig,
}

impl Default for Glyphr {
    /// Create a new text renderer with default configuration.
    fn default() -> Self {
        Self::new()
    }
}

impl Glyphr {
    /// Create a new text renderer with default configuration.
    pub fn new() -> Self {
        Self {
            render_config: RenderConfig::default(),
        }
    }

    /// Create a new text renderer with custom configuration.
    pub fn with_config(render_config: RenderConfig) -> Self {
        Self { render_config }
    }

    /// Update the render configuration.
    pub fn set_config(&mut self, config: RenderConfig) {
        self.render_config = config;
    }

    /// Get the current render configuration.
    pub fn config(&self) -> &RenderConfig {
        &self.render_config
    }

    /// Render text to any target that implements RenderTarget.
    pub fn render<T: RenderTarget>(
        &self,
        target: &mut T,
        text: &str,
        font: Font,
        mut x: i32,
        y: i32,
        align: TextAlign,
    ) -> Result<(), GlyphrError> {
        let scale = match font.format {
            BitmapFormat::SDF => self.render_config.sdf.size as f32 / font.size as f32,
            BitmapFormat::Bitmap => 1.0,
        };
        let ascent = font.ascent;
        let descent = font.descent;

        let x_offset = match align.horizontal {
            AlignH::Center => self.phrase_length(text, font) / 2,
            AlignH::Right => self.phrase_length(text, font),
            AlignH::Left => 0,
        };

        let y_offset = match align.vertical {
            AlignV::Top => (descent as f32 * scale) as i32,
            AlignV::Center => {
                let total_height = (ascent - descent) as f32 * scale;
                -(total_height / 2.0) as i32
            }
            AlignV::Baseline => -(ascent as f32 * scale) as i32,
        };

        for c in text.chars() {
            let glyph = font.find_glyph(c)?;
            let glyph_y =
                y + y_offset + ((ascent - glyph.ymin - glyph.height) as f32 * scale) as i32;
            renderer::render_glyph(x - x_offset, glyph_y, c, font, self, scale, target)?;
            x += (renderer::advance(c, font).unwrap_or(0) as f32 * scale) as i32;
        }

        Ok(())
    }

    /// Returns the lenght of the string that will be rendered.
    pub fn phrase_length(&self, phrase: &str, font: Font) -> i32 {
        let scale = match font.format {
            BitmapFormat::SDF => self.render_config.sdf.size as f32 / font.size as f32,
            BitmapFormat::Bitmap => 1.0,
        };
        let mut tot = 0;
        for c in phrase.chars() {
            tot += (renderer::advance(c, font).unwrap_or(0) as f32 * scale) as i32;
        }
        tot
    }
}

#[derive(Debug, Clone)]
pub enum GlyphrError {
    OutOfBounds,
    InvalidGlyph(char),
    InvalidTarget,
}

impl core::fmt::Display for GlyphrError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GlyphrError::OutOfBounds => write!(f, "Rendering position is out of bounds"),
            GlyphrError::InvalidGlyph(c) => write!(f, "Glyph not found: '{c}'"),
            GlyphrError::InvalidTarget => write!(f, "Invalid render target"),
        }
    }
}

impl core::error::Error for GlyphrError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdf_config_default_values() {
        let cfg = SdfConfig::default();
        assert_eq!(cfg.size, 16);
        assert_eq!(cfg.mid_value, 0.5);
        assert_eq!(cfg.smoothing, 0.1);
    }

    #[test]
    fn test_render_config_default_values() {
        let cfg = RenderConfig::default();
        assert_eq!(cfg.color, 0xffffff);
        assert_eq!(cfg.sdf.size, 16);
        assert_eq!(cfg.sdf.mid_value, 0.5);
        assert_eq!(cfg.sdf.smoothing, 0.1);
    }

    #[test]
    fn test_glyphr_new_initializes_correctly() {
        let glyphr = Glyphr::new();

        assert_eq!(glyphr.render_config.color, 0xffffff);
        assert_eq!(glyphr.render_config.sdf.size, 16);
        assert_eq!(glyphr.render_config.sdf.mid_value, 0.5);
        assert_eq!(glyphr.render_config.sdf.smoothing, 0.1);
    }

    #[test]
    fn test_pixel_callback_writes_color() {
        let mut buffer = [0u32; 16];
        let mut target = BufferTarget::new(&mut buffer, 4, 4);
        target.write_pixel(2, 1, 0xff123456);

        let idx = 1 * 4 + 2;
        assert_eq!(buffer[idx], 0xff123456);
    }

    #[test]
    fn test_buffer_target_dimentsions() {
        let mut buffer = [0u32; 16];
        let target = BufferTarget::new(&mut buffer, 4, 4);

        assert_eq!(target.dimensions(), (4, 4));
    }
}
