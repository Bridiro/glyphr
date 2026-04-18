use glyphr::{AlignH, AlignV, Callbacks, Glyphr, RenderConfig, SdfConfig, TextAlign};
#[cfg(feature = "window")]
use minifb::{Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 480;

fn main() {
    let mut buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

    #[cfg(feature = "window")]
    let mut window = Window::new(
        "Pixel Buffer Test",
        WIDTH,
        HEIGHT,
        WindowOptions {
            ..WindowOptions::default()
        },
    )
    .expect("Failed to create window");

    for x in 0..WIDTH {
        buffer[120 * WIDTH + x] = 0xffffffff;
        buffer[240 * WIDTH + x] = 0xffffffff;
        buffer[360 * WIDTH + x] = 0xffffffff;
    }

    let conf = RenderConfig::default()
        .with_color(0x00ff_ffff)
        .with_sdf(SdfConfig::default().with_size(64).with_smoothing(0.5));
    let renderer = Glyphr::with_config(conf);

    let mut target = Callbacks::new(WIDTH as u16, HEIGHT as u16, |x, y, color| {
        let index = y as usize * WIDTH + x as usize;
        if index >= buffer.len() {
            return false;
        }

        // Alpha blend directly in callback so users can plug custom accelerators.
        let bg = buffer[index];
        let alpha = ((color >> 24) & 0xff) as u8;
        if alpha == 0xff {
            buffer[index] = color;
            return true;
        }

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
        true
    });

    glyphr::generate_font! {
        name: POPPINS_BITMAP,
        path: "fonts/Poppins-Regular.ttf",
        size: 64,
        characters: "A-Za-z! ",
        format: Bitmap {
            spread: 10.0,
            padding: 0,
        },
    }

    glyphr::generate_font! {
        name: POPPINS_SDF,
        path: "fonts/Poppins-Regular.ttf",
        size: 64,
        characters: "A-Za-z! ",
        format: SDF {
            spread: 20.0,
            padding: 0,
        },
    }

    glyphr::generate_fonts_from_toml!("fonts/fonts.toml");

    renderer
        .draw_text(
            &mut target,
            "TEST base left!",
            POPPINS_TOML,
            0,
            120,
            TextAlign::new(AlignH::Left, AlignV::Baseline),
        )
        .unwrap();

    renderer
        .draw_text(
            &mut target,
            "TEST center center!",
            POPPINS_BITMAP,
            400,
            240,
            TextAlign::new(AlignH::Center, AlignV::Center),
        )
        .unwrap();

    renderer
        .draw_text(
            &mut target,
            "TEST top right!",
            POPPINS_SDF,
            800,
            360,
            TextAlign::new(AlignH::Right, AlignV::Top),
        )
        .unwrap();

    drop(target);

    #[cfg(feature = "window")]
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
