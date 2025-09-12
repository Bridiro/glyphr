use glyphr::{AlignH, AlignV, BufferTarget, Glyphr, RenderConfig, SdfConfig, TextAlign};
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

    let mut target = BufferTarget::new(&mut buffer, 200, 200);
    let conf = RenderConfig {
        color: 0xffffff,
        sdf: SdfConfig {
            size: 128,
            mid_value: 0.5,
            smoothing: 0.5,
        },
    };
    let renderer = Glyphr::with_config(conf);

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
        .render(
            &mut target,
            "Aa",
            FONT,
            100,
            100,
            TextAlign {
                horizontal: AlignH::Center,
                vertical: AlignV::Center,
            },
        )
        .unwrap();

    #[cfg(feature = "window")]
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
