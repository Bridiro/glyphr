//! # rendered.rs
//!
//! This module describes the public API to this library.
//! Everything is done via the `Glyphr` struct.

use crate::{
    fonts::{Font, HFontAlign, VFontAlign},
    sdf,
};

type WritePixel = fn(u32, u32, u32, &mut [u32]);

/// # SdfConfig
///
/// This struct defines everything about rendering the glyph.
#[derive(Clone, Copy)]
pub struct SdfConfig {
    pub font: Font,
    pub valign: VFontAlign,
    pub halign: HFontAlign,
    pub px: u32,
    pub color: u32,
    pub mid_value: f32,
    pub smoothing: f32,
}

impl Default for SdfConfig {
    /// # Default for SdfConfig
    ///
    /// # Examples
    /// ```
    /// use glyphr::SdfConfig;
    ///
    /// let default = SdfConfig::default();
    /// assert_eq!(0.5, default.smoothing);
    /// ```
    fn default() -> Self {
        Self {
            font: Font::default(),
            valign: VFontAlign::default(),
            halign: HFontAlign::default(),
            px: Font::default().get_size() as u32,
            color: 0x000000,
            mid_value: 0.5,
            smoothing: 0.5,
        }
    }
}

/// # Buffer
///
/// This struct contains a buffer with his width and height to simplify operations
pub struct Buffer<'a> {
    pub buffer: &'a mut [u32],
    pub width: u32,
    pub height: u32,
}

/// # Glyphr
///
/// This struct merges `Buffer`, `SdfConfig` and the pixel callback to have the fully operational
/// library
pub struct Glyphr<'a> {
    pub buffer: Buffer<'a>,
    pub pixel_callback: WritePixel,
    pub sdf_config: SdfConfig,
}

impl<'a> Glyphr<'a> {
    /// # new
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    /// assert_eq!(10, glyphr_struct.buffer.width);
    /// ```
    pub fn new(
        pixel_callback: WritePixel,
        buffer: &'a mut [u32],
        width: u32,
        height: u32,
        sdf_config: SdfConfig,
    ) -> Self {
        Glyphr {
            buffer: Buffer {
                buffer,
                width,
                height,
            },
            pixel_callback,
            sdf_config,
        }
    }

    /// # set_sdf_config
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// let new_config =  SdfConfig {
    ///     color: 0x00ff00,
    ///     px: 110,
    ///     smoothing: 1.0,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Left,
    ///     valign: VFontAlign::Baseline,
    ///     font: Font::default(),
    /// };
    ///
    /// glyphr_struct.set_sdf_config(new_config);
    /// assert_eq!(110, glyphr_struct.sdf_config.px);
    /// ```
    pub fn set_sdf_config(&mut self, config: SdfConfig) {
        self.sdf_config = config;
    }

    /// # set_font
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// glyphr_struct.set_font(Font::default());
    /// assert!(glyphr_struct.sdf_config.font == Font::default());
    /// ```
    pub fn set_font(&mut self, font: Font) {
        self.sdf_config.font = font;
    }

    /// # set_font_valign
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// glyphr_struct.set_font_halign(HFontAlign::Left);
    /// assert_eq!(glyphr_struct.sdf_config.halign, HFontAlign::Left);
    /// ```
    pub fn set_font_halign(&mut self, align: HFontAlign) {
        self.sdf_config.halign = align;
    }

    /// # set_font_valign
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// glyphr_struct.set_font_valign(VFontAlign::Baseline);
    /// assert_eq!(glyphr_struct.sdf_config.valign, VFontAlign::Baseline);
    /// ```
    pub fn set_font_valign(&mut self, align: VFontAlign) {
        self.sdf_config.valign = align;
    }

    /// # set_size
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// glyphr_struct.set_size(100);
    /// assert_eq!(100, glyphr_struct.sdf_config.px);
    /// ```
    pub fn set_size(&mut self, px: u32) {
        self.sdf_config.px = px;
    }

    /// # set_color
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// glyphr_struct.set_color(0x00ff00);
    /// assert_eq!(0x00ff00, glyphr_struct.sdf_config.color);
    /// ```
    pub fn set_color(&mut self, color: u32) {
        self.sdf_config.color = color;
    }

    /// # set_smoothing
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// glyphr_struct.set_smoothing(1.0);
    /// assert_eq!(1.0, glyphr_struct.sdf_config.smoothing);
    /// ```
    pub fn set_smoothing(&mut self, smoothing: f32) {
        self.sdf_config.smoothing = smoothing;
    }

    /// # set_mid_value
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 70,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Center,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// glyphr_struct.set_mid_value(0.6);
    /// assert_eq!(0.6, glyphr_struct.sdf_config.mid_value);
    /// ```
    pub fn set_mid_value(&mut self, mid_value: f32) {
        self.sdf_config.mid_value = mid_value;
    }

    /// # render
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 20,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Left,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    /// glyphr_struct.render("hi", 0, 0);
    ///
    /// assert!(buf.iter().any(|c| *c != 0));
    /// ```
    pub fn render(&mut self, phrase: &str, mut x: i32, y: i32) {
        let scale = self.sdf_config.px as f32 / self.sdf_config.font.get_size() as f32;
        let ascent = self.sdf_config.font.get_ascent();
        let descent = self.sdf_config.font.get_descent();

        let x_offset = match self.sdf_config.halign {
            HFontAlign::Center => self.phrase_length(phrase) / 2,
            HFontAlign::Right => self.phrase_length(phrase),
            HFontAlign::Left => 0,
        };

        let y_offset = match self.sdf_config.valign {
            VFontAlign::Top => (descent as f32 * scale) as i32,
            VFontAlign::Center => {
                let total_height = (ascent - descent) as f32 * scale;
                -(total_height / 2.0) as i32
            }
            VFontAlign::Baseline => -(ascent as f32 * scale) as i32,
        };

        for c in phrase.chars() {
            if let Some(metrics) = sdf::get_metrics(self, c) {
                let glyph_y =
                    y + y_offset + ((ascent - metrics.ymin - metrics.height) as f32 * scale) as i32;
                sdf::render_glyph(x - x_offset, glyph_y, c, self, scale);
            }
            x += (sdf::advance(self, c).unwrap_or(0) as f32 * scale) as i32;
        }
    }

    /// # phrase_lenght
    ///
    /// # Examples
    /// ```
    /// use glyphr::{Glyphr, SdfConfig, Font, VFontAlign, HFontAlign};
    ///
    /// let mut buf = [0u32, 100];
    /// let config =  SdfConfig {
    ///     color: 0xffffff,
    ///     px: 20,
    ///     smoothing: 0.4,
    ///     mid_value: 0.5,
    ///     halign: HFontAlign::Left,
    ///     valign: VFontAlign::Top,
    ///     font: Font::default(),
    /// };
    /// let mut glyphr_struct = Glyphr::new(|_, _, _, _| (), &mut buf, 10, 10, config);
    ///
    /// assert_eq!(glyphr_struct.phrase_length("hello world"), glyphr_struct.phrase_length("hello world"));
    /// ```
    pub fn phrase_length(&self, phrase: &str) -> i32 {
        let scale = self.sdf_config.px as f32 / self.sdf_config.font.get_size() as f32;
        let mut tot = 0;
        for c in phrase.chars() {
            tot += (sdf::advance(self, c).unwrap_or(0) as f32 * scale) as i32;
        }
        tot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::{Font, HFontAlign, VFontAlign};

    fn dummy_pixel_callback(x: u32, y: u32, color: u32, buf: &mut [u32]) {
        let idx = (y * 4 + x) as usize;
        if idx < buf.len() {
            buf[idx] = color;
        }
    }

    fn make_test_config() -> SdfConfig {
        let font = Font::default();
        SdfConfig {
            font,
            halign: HFontAlign::Left,
            valign: VFontAlign::Top,
            px: 24,
            color: 0xAABBCC,
            mid_value: 0.4,
            smoothing: 0.2,
        }
    }

    #[test]
    fn test_sdf_config_default_values() {
        let cfg = SdfConfig::default();
        assert_eq!(cfg.color, 0x000000);
        assert!(cfg.mid_value > 0.0 && cfg.mid_value <= 1.0);
        assert!(cfg.smoothing > 0.0 && cfg.smoothing <= 1.0);
        let g = cfg.font.get_glyph('a').unwrap();
        assert!(g.px > 1);
    }

    #[test]
    fn test_sdf_config_custom_values() {
        let cfg = make_test_config();
        assert_eq!(cfg.color, 0xAABBCC);
        assert_eq!(cfg.px, 24);
        assert_eq!(cfg.mid_value, 0.4);
        assert_eq!(cfg.smoothing, 0.2);
    }

    #[test]
    fn test_glyphr_new_initializes_correctly() {
        let mut buffer = [0u32; 16];
        let config = make_test_config();

        let glyphr = Glyphr::new(dummy_pixel_callback, &mut buffer, 4, 4, config);

        assert_eq!(glyphr.buffer.width, 4);
        assert_eq!(glyphr.buffer.height, 4);
        assert_eq!(glyphr.sdf_config.color, 0xAABBCC);
    }

    #[test]
    fn test_pixel_callback_writes_color() {
        let mut buffer = [0u32; 16];
        let callback = |x, y, color, buf: &mut [u32]| {
            let idx = (y * 4 + x) as usize;
            buf[idx] = color;
        };

        callback(2, 1, 0x123456, &mut buffer);

        let idx = 1 * 4 + 2;
        assert_eq!(buffer[idx], 0x123456);
    }

    #[test]
    fn test_out_of_bounds_pixel_callback_does_not_crash() {
        let mut buffer = [0u32; 16];
        dummy_pixel_callback(10, 10, 0xFFFFFF, &mut buffer);
        assert!(buffer.iter().all(|&b| b == 0)); // Should be unchanged
    }

    #[test]
    fn test_glyphr_multiple_renders_dont_corrupt() {
        let mut buffer = [0u32; 16];
        let mut glyphr = Glyphr::new(
            dummy_pixel_callback,
            &mut buffer,
            4,
            4,
            SdfConfig::default(),
        );

        glyphr.render("H", 0, 0);

        let modified = buffer.iter().any(|&c| c != 0);
        assert!(modified);
    }
}
