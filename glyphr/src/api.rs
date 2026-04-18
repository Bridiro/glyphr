//! # api.rs
//!
//! Public API for text rendering.

use crate::renderer;
use glyphr_types::{AlignH, AlignV, BitmapFormat, Font};

/// Callback-backed render target.
pub struct Callbacks<P>
where
    P: FnMut(u16, u16, u32) -> bool,
{
    width: u16,
    height: u16,
    write_pixel: P,
}

impl<P> Callbacks<P>
where
    P: FnMut(u16, u16, u32) -> bool,
{
    pub fn new(width: u16, height: u16, write_pixel: P) -> Self {
        Self {
            width,
            height,
            write_pixel,
        }
    }

    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn write_pixel(&mut self, x: u16, y: u16, color: u32) -> bool {
        (self.write_pixel)(x, y, color)
    }
}

/// Configuration for text rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderConfig {
    /// Color to render the text.
    pub color: u32,
    /// SDF-specific configuration (ignored for bitmap fonts).
    pub sdf: SdfConfig,
}

impl RenderConfig {
    pub const fn with_color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    pub const fn with_sdf(mut self, sdf: SdfConfig) -> Self {
        self.sdf = sdf;
        self
    }
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfConfig {
    /// Font size in pixels (only affects SDF fonts).
    pub size: u16,
    /// Mid-value for SDF (usually 0.5).
    pub mid_value: f32,
    /// Smoothing factor for anti-aliasing.
    pub smoothing: f32,
}

impl SdfConfig {
    pub const fn with_size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    pub const fn with_mid_value(mut self, mid_value: f32) -> Self {
        self.mid_value = mid_value;
        self
    }

    pub const fn with_smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = smoothing;
        self
    }
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextAlign {
    pub horizontal: AlignH,
    pub vertical: AlignV,
}

impl TextAlign {
    pub const fn new(horizontal: AlignH, vertical: AlignV) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

impl Default for TextAlign {
    fn default() -> Self {
        Self {
            horizontal: AlignH::Left,
            vertical: AlignV::Top,
        }
    }
}

/// Main renderer struct.
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

    /// Renders text using the configured per-pixel callback.
    pub fn draw_text<P>(
        &self,
        target: &mut Callbacks<P>,
        text: &str,
        font: Font,
        mut x: i32,
        y: i32,
        align: TextAlign,
    ) -> Result<(), GlyphrError>
    where
        P: FnMut(u16, u16, u32) -> bool,
    {
        let scale = match font.format {
            BitmapFormat::SDF => self.render_config.sdf.size as f32 / font.size as f32,
            BitmapFormat::Bitmap => 1.0,
        };
        let ascent = i32::from(font.ascent);
        let descent = i32::from(font.descent);

        let x_offset = match align.horizontal {
            AlignH::Center => self.measure_text(text, font) / 2,
            AlignH::Right => self.measure_text(text, font),
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
            let glyph = font.find_glyph(c).ok_or(GlyphrError::InvalidGlyph(c))?;
            let glyph_y = y
                + y_offset
                + ((ascent - i32::from(glyph.ymin) - i32::from(glyph.height)) as f32 * scale)
                    as i32;
            renderer::render_glyph(x - x_offset, glyph_y, c, font, self, scale, target)?;
            x += (glyph.advance_width as f32 * scale) as i32;
        }

        Ok(())
    }

    /// Returns the width of text in pixels for the current render config.
    pub fn measure_text(&self, phrase: &str, font: Font) -> i32 {
        let scale = match font.format {
            BitmapFormat::SDF => self.render_config.sdf.size as f32 / font.size as f32,
            BitmapFormat::Bitmap => 1.0,
        };
        let mut tot = 0;
        for c in phrase.chars() {
            if let Some(glyph) = font.find_glyph(c) {
                tot += (glyph.advance_width as f32 * scale) as i32;
            }
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
    fn test_callbacks_sink_works() {
        let mut writes = 0u32;
        let mut target = Callbacks::new(8, 8, |_x, _y, _color| {
            writes += 1;
            true
        });

        assert!(target.write_pixel(1, 1, 0xff00ff00));
        assert_eq!(writes, 1);
    }
}
