extern crate glyphr;

use minifb::{Window, WindowOptions};
use glyphr::Glyphr;

fn test_pixel_buffer_with_window() {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 480;

    let mut buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

    fn put_pixel(x: u32, y: u32, color: u32, buffer: &mut [u32]) {
        buffer[(y as usize) * WIDTH + (x as usize)] = color;
    }

    let mut window = Window::new("Pixel Buffer Test", WIDTH, HEIGHT, WindowOptions {
        ..WindowOptions::default()
    })
    .expect("Failed to create window");

    let mut current = Glyphr::new(put_pixel, &mut buffer, WIDTH as u32, HEIGHT as u32);
    current.render("test up & down!", 50, 50, 1.0, 0.5, 0.3, 0x00ffffff);

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn main() {
    test_pixel_buffer_with_window();
}
