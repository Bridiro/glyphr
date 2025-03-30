
mod fonts;
mod sdf;

type WritePixel = fn(u32, u32, u32, &mut [u32]);

pub struct Buffer<'a> {
    buffer: &'a mut [u32],
    width: u32,
    height: u32,
}

pub struct Glyphr<'a> {
    current_font: &'static [fonts::GlyphEntry],
    buffer: Buffer<'a>,
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

    let mut current = Glyphr::new(put_pixel, &mut buffer, WIDTH as u32, HEIGHT as u32);
    current.render("a", 100, 100, 6.0, 0.5, 0.3, 0x00ffffff);

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}


#[cfg(test)]
mod tests {
}
