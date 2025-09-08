use glyphr::{AlignH, AlignV, BufferTarget, Glyphr, RenderConfig, SdfConfig, TextAlign};
#[cfg(feature = "window")]
use minifb::{Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 480;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

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

    let mut target = BufferTarget::new(&mut buffer, 800, 480);
    let conf = RenderConfig {
        color: 0xffffff,
        sdf: SdfConfig {
            size: 64,
            mid_value: 0.5,
            smoothing: 0.5,
        },
    };
    let renderer = Glyphr::with_config(conf);

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
        .render(
            &mut target,
            "TEST base left!",
            POPPINS_TOML,
            0,
            120,
            TextAlign {
                horizontal: AlignH::Left,
                vertical: AlignV::Baseline,
            },
        )
        .unwrap();

    renderer
        .render(
            &mut target,
            "TEST center center!",
            POPPINS_BITMAP,
            400,
            240,
            TextAlign {
                horizontal: AlignH::Center,
                vertical: AlignV::Center,
            },
        )
        .unwrap();

    renderer
        .render(
            &mut target,
            "TEST top right!",
            POPPINS_SDF,
            800,
            360,
            TextAlign {
                horizontal: AlignH::Right,
                vertical: AlignV::Top,
            },
        )
        .unwrap();

    #[cfg(feature = "window")]
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
