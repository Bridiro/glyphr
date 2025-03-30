#![no_std]

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



#[cfg(test)]
mod tests {
}
