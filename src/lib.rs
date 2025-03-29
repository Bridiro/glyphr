#![no_std]

mod fonts;
mod sdf;

type WritePixel = fn(u32, u32, u32, &mut [u32]);

pub struct Glyphr<'a> {
    current_font: &'static [u8],
    buffer: &'a mut [u32],
    pixel_callback: WritePixel,
}

use minifb::{Window, WindowOptions};

pub fn test_pixel_buffer_with_window() {
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

    let mut current = Glyphr::new(put_pixel, &mut buffer);
    current.render("a", 4.0, 0.5, 15.0, 0xffffffff);

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(current.get_buffer(), WIDTH, HEIGHT).unwrap();
    }
}


#[cfg(test)]
mod tests {
}
