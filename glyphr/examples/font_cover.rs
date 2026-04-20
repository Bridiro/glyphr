use glyphr::{AlignH, AlignV, Callbacks, Glyphr, RenderConfig, SdfConfig, TextAlign};
#[cfg(feature = "window")]
use minifb::{Window, WindowOptions};

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

fn main() {
    let mut buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

    #[cfg(feature = "window")]
    let mut window = Window::new(
        "Font Cover",
        WIDTH,
        HEIGHT,
        WindowOptions {
            ..WindowOptions::default()
        },
    )
    .expect("Failed to create window");

    let conf = RenderConfig::default()
        .with_color(0x00ff_ffff)
        .with_sdf(SdfConfig::default().with_size(128).with_smoothing(0.5));
    let renderer = Glyphr::with_config(conf);

    let mut target = Callbacks::new(WIDTH as u16, HEIGHT as u16, |x, y, color| {
        let index = y as usize * WIDTH + x as usize;
        if index >= buffer.len() {
            return false;
        }

        let alpha = ((color >> 24) & 0xff) as u8;
        if alpha == 0xff {
            buffer[index] = color;
        } else {
            let bg = buffer[index];
            let a = alpha as f32 / 255.0;
            let fg_r = (color >> 16) & 0xff;
            let fg_g = (color >> 8) & 0xff;
            let fg_b = color & 0xff;
            let bg_r = (bg >> 16) & 0xff;
            let bg_g = (bg >> 8) & 0xff;
            let bg_b = bg & 0xff;
            let out_r = ((fg_r as f32 * a) + (bg_r as f32 * (1.0 - a))) as u32;
            let out_g = ((fg_g as f32 * a) + (bg_g as f32 * (1.0 - a))) as u32;
            let out_b = ((fg_b as f32 * a) + (bg_b as f32 * (1.0 - a))) as u32;
            buffer[index] = (0xff << 24) | (out_r << 16) | (out_g << 8) | out_b;
        }

        true
    });

    glyphr::generate_font! {
        name: FONT,
        path: "fonts/Poppins-Regular.ttf",
        size: 128,
        characters: "A-Za-z! ",
        format: SDF {
            spread: 20.0,
            padding: 0,
        },
    }

    renderer
        .draw_text(
            &mut target,
            "Aa",
            FONT,
            100,
            100,
            TextAlign::new(AlignH::Center, AlignV::Center),
        )
        .unwrap();

    drop(target);

    #[cfg(feature = "window")]
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
