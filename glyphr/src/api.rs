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
    /// Width of the render target in pixels.
    width: u16,
    /// Height of the render target in pixels.
    height: u16,
    /// Callback to write a single pixel: (x, y, color) -> success.
    write_pixel: P,
}

impl<P> Callbacks<P>
where
    P: FnMut(u16, u16, u32) -> bool,
{
    /// Create a new callback render target.
    pub fn new(width: u16, height: u16, write_pixel: P) -> Self {
        Self {
            width,
            height,
            write_pixel,
        }
    }

    /// Get the dimensions of the render target.
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Write a pixel to the render target using the callback.
    pub fn write_pixel(&mut self, x: u16, y: u16, color: u32) -> bool {
        (self.write_pixel)(x, y, color)
    }
}

/// Callback-backed render target for per-glyph bulk writes.
pub struct BulkCallbacks<'a, P>
where
    P: FnMut(i32, i32, u16, u16, &[u32]) -> bool,
{
    /// Width of the render target in pixels.
    width: u16,
    /// Height of the render target in pixels.
    height: u16,
    /// Scratch buffer for glyph rendering (must be large enough to hold the largest glyph).
    scratch: &'a mut [u32],
    /// Callback to write a glyph: (x, y, width, height, glyph) -> success.
    write_glyph: P,
}

impl<'a, P> BulkCallbacks<'a, P>
where
    P: FnMut(i32, i32, u16, u16, &[u32]) -> bool,
{
    /// Create a new bulk callback render target.
    pub fn new(width: u16, height: u16, scratch: &'a mut [u32], write_glyph: P) -> Self {
        Self {
            width,
            height,
            scratch,
            write_glyph,
        }
    }

    /// Get the dimensions of the render target.
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Get the capacity of the scratch buffer in pixels.
    pub fn scratch_capacity(&self) -> usize {
        self.scratch.len()
    }

    /// Get a mutable reference to the scratch buffer for glyph rendering.
    pub fn scratch_mut(&mut self) -> &mut [u32] {
        self.scratch
    }

    /// Write a glyph to the render target using the callback.
    pub fn write_glyph(&mut self, x: i32, y: i32, width: u16, height: u16, pixels: &[u32]) -> bool {
        (self.write_glyph)(x, y, width, height, pixels)
    }

    /// Internal helper to write from the scratch buffer, ensuring we don't exceed its capacity.
    pub(crate) fn emit_from_scratch(
        &mut self,
        x: i32,
        y: i32,
        width: u16,
        height: u16,
        used_pixels: usize,
    ) -> bool {
        let pixels = &self.scratch[..used_pixels];
        (self.write_glyph)(x, y, width, height, pixels)
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
    /// Create a new render config with the specified color.
    pub const fn with_color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    /// Create a new render config with the specified SDF settings.
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
    /// Create a new SDF config with the specified size.
    pub const fn with_size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Create a new SDF config with the specified mid-value.
    pub const fn with_mid_value(mut self, mid_value: f32) -> Self {
        self.mid_value = mid_value;
        self
    }

    /// Create a new SDF config with the specified smoothing factor.
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
    /// Horizontal alignment.
    pub horizontal: AlignH,
    /// Vertical alignment.
    pub vertical: AlignV,
}

impl TextAlign {
    /// Create a new TextAlign with the specified horizontal and vertical alignment.
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

    /// Renders text calling the callback once per glyph with a full ARGB buffer.
    pub fn draw_text_bulk<P>(
        &self,
        target: &mut BulkCallbacks<'_, P>,
        text: &str,
        font: Font,
        mut x: i32,
        y: i32,
        align: TextAlign,
    ) -> Result<(), GlyphrError>
    where
        P: FnMut(i32, i32, u16, u16, &[u32]) -> bool,
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
            renderer::render_glyph_bulk(x - x_offset, glyph_y, c, font, self, scale, target)?;
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
    /// Rendering position is out of bounds of the target.
    OutOfBounds,
    /// The font does not contain a glyph for the specified character.
    InvalidGlyph(char),
    /// The provided scratch buffer is too small to render the largest glyph.
    BufferTooSmall { needed: usize, available: usize },
    /// The render target is invalid (e.g. callback returned false).
    InvalidTarget,
}

impl core::fmt::Display for GlyphrError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GlyphrError::OutOfBounds => write!(f, "Rendering position is out of bounds"),
            GlyphrError::InvalidGlyph(c) => write!(f, "Glyph not found: '{c}'"),
            GlyphrError::BufferTooSmall { needed, available } => {
                write!(
                    f,
                    "Scratch buffer too small: needed {needed} pixels, available {available}"
                )
            }
            GlyphrError::InvalidTarget => write!(f, "Invalid render target"),
        }
    }
}

impl core::error::Error for GlyphrError {}

#[cfg(test)]
mod tests {
    use super::*;
    use glyphr_types::Glyph;

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

    #[test]
    fn test_bulk_callbacks_sink_works() {
        let mut writes = 0u32;
        let mut scratch = [0u32; 16];
        let mut target = BulkCallbacks::new(8, 8, &mut scratch, |_x, _y, w, h, pixels| {
            writes += 1;
            assert_eq!(w, 2);
            assert_eq!(h, 2);
            assert_eq!(pixels.len(), 4);
            true
        });

        let tile = [0xff00ff00u32; 4];
        assert!(target.write_glyph(1, 1, 2, 2, &tile));
        assert_eq!(writes, 1);
    }

    #[test]
    fn test_draw_text_bulk_errors_when_scratch_too_small() {
        let glyph_bitmap = [0b1111_0000u8];
        let glyphs = [Glyph {
            character: 'A',
            bitmap: &glyph_bitmap,
            width: 2,
            height: 2,
            xmin: 0,
            ymin: 0,
            advance_width: 2,
        }];

        let font = Font {
            glyphs: &glyphs,
            size: 16,
            ascent: 2,
            descent: 0,
            line_gap: 0,
            format: BitmapFormat::Bitmap,
        };

        let glyphr = Glyphr::new();
        let mut scratch = [0u32; 3];
        let mut target = BulkCallbacks::new(16, 16, &mut scratch, |_x, _y, _w, _h, _pixels| true);

        let result = glyphr.draw_text_bulk(
            &mut target,
            "A",
            font,
            0,
            0,
            TextAlign::new(AlignH::Left, AlignV::Top),
        );

        assert!(matches!(
            result,
            Err(GlyphrError::BufferTooSmall {
                needed: 4,
                available: 3
            })
        ));
    }
}
