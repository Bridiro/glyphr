use glyphr::{AlignH, AlignV, BulkCallbacks, Glyphr, RenderConfig, SdfConfig, TextAlign};
#[cfg(feature = "window")]
use minifb::{Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 480;

fn main() {
    let mut buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

    #[cfg(feature = "window")]
    let mut window = Window::new(
        "Glyphr Bulk Callback",
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

    // Size this for the largest expected rendered glyph tile.
    let mut scratch = [0u32; 128 * 128];
    let mut target = BulkCallbacks::new(
        WIDTH as u16,
        HEIGHT as u16,
        &mut scratch,
        |x, y, w, h, pixels| {
            let w_usize = w as usize;
            let h_usize = h as usize;
            if pixels.len() != w_usize * h_usize {
                return false;
            }

            for gy in 0..h_usize {
                let dst_y = y + gy as i32;
                if !(0..HEIGHT as i32).contains(&dst_y) {
                    continue;
                }

                for gx in 0..w_usize {
                    let dst_x = x + gx as i32;
                    if !(0..WIDTH as i32).contains(&dst_x) {
                        continue;
                    }

                    let src = pixels[gy * w_usize + gx];
                    let alpha = ((src >> 24) & 0xff) as u8;
                    if alpha == 0 {
                        continue;
                    }

                    let dst_index = dst_y as usize * WIDTH + dst_x as usize;
                    if alpha == 0xff {
                        buffer[dst_index] = src;
                        continue;
                    }

                    let bg = buffer[dst_index];
                    let a = alpha as f32 / 255.0;

                    let fg_r = (src >> 16) & 0xff;
                    let fg_g = (src >> 8) & 0xff;
                    let fg_b = src & 0xff;

                    let bg_r = (bg >> 16) & 0xff;
                    let bg_g = (bg >> 8) & 0xff;
                    let bg_b = bg & 0xff;

                    let out_r = ((fg_r as f32 * a) + (bg_r as f32 * (1.0 - a))) as u32;
                    let out_g = ((fg_g as f32 * a) + (bg_g as f32 * (1.0 - a))) as u32;
                    let out_b = ((fg_b as f32 * a) + (bg_b as f32 * (1.0 - a))) as u32;

                    buffer[dst_index] = (0xff << 24) | (out_r << 16) | (out_g << 8) | out_b;
                }
            }

            true
        },
    );

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

    renderer
        .draw_text_bulk(
            &mut target,
            "BULK base left!",
            POPPINS_BITMAP,
            0,
            120,
            TextAlign::new(AlignH::Left, AlignV::Baseline),
        )
        .unwrap();

    renderer
        .draw_text_bulk(
            &mut target,
            "BULK center center!",
            POPPINS_BITMAP,
            400,
            240,
            TextAlign::new(AlignH::Center, AlignV::Center),
        )
        .unwrap();

    renderer
        .draw_text_bulk(
            &mut target,
            "BULK top right!",
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
