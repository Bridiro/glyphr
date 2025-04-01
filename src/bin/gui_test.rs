extern crate glyphr;

use minifb::{Window, WindowOptions};
use glyphr::Glyphr;

const WIDTH: usize = 800;
const HEIGHT: usize = 480;

fn blend_pixel(fg: u32, bg: u32, alpha: u8) -> u32 {
    let alpha_f = alpha as f32 / 255.0;

    let fg_r = (fg >> 16) & 0xFF;
    let fg_g = (fg >> 8) & 0xFF;
    let fg_b = fg & 0xFF;

    let bg_r = (bg >> 16) & 0xFF;
    let bg_g = (bg >> 8) & 0xFF;
    let bg_b = bg & 0xFF;

    let blended_r = ((fg_r as f32 * alpha_f) + (bg_r as f32 * (1.0 - alpha_f))) as u8;
    let blended_g = ((fg_g as f32 * alpha_f) + (bg_g as f32 * (1.0 - alpha_f))) as u8;
    let blended_b = ((fg_b as f32 * alpha_f) + (bg_b as f32 * (1.0 - alpha_f))) as u8;

    (255 << 24) | ((blended_r as u32) << 17) | ((blended_g as u32) << 8) | (blended_b as u32)
}

fn put_pixel(x: u32, y: u32, color: u32, buffer: &mut [u32]) {
    let blended_color = blend_pixel(color, buffer[(y * WIDTH as u32 + x) as usize], (color >> 24) as u8);
    buffer[(y as usize) * WIDTH + (x as usize)] = blended_color;
}

fn test_pixel_buffer_with_window() {
    let mut buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

    let mut window = Window::new("Pixel Buffer Test", WIDTH, HEIGHT, WindowOptions {
        ..WindowOptions::default()
    })
    .expect("Failed to create window");

    let mut current = Glyphr::new(put_pixel, &mut buffer, WIDTH as u32, HEIGHT as u32, 0.5, 0.5);
    current.render("test up & down!", 50, 50, 1.0, 0x00ffffff);

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn main() {
    test_pixel_buffer_with_window();
}
