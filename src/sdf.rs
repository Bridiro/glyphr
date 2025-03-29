use super::fonts;

pub fn render(
    scale: f32,
    mid_value: f32,
    smoothing: f32,
    sdf: fonts::GlyphEntry,
) {
    let width = (sdf.metrics.width as f32 * scale) as u32;
    let height = (sdf.metrics.height as f32 * scale) as u32;
    if width <= 0 || height <= 0 {
        panic!("Scaling of {:?} returns an image size of {:?}, which is impossible to render", scale, (width, height));
    }

    let width_f = width as f32;
    let height_f = height as f32;

    let distance_to_pixel = |distance: f32| {
        match distance > mid_value {
            true => (smoothstep(mid_value-smoothing, mid_value+smoothing, distance) * 255.0) as u8,
            false => 0,
        }
    };

    for x in 0..width {
        for y in 0..height {
            let sample_x = ((x as f32) + 0.5) / width_f;
            let sample_y = ((y as f32) + 0.5) / height_f;

            let sampled_distance = sdf_sample(&sdf, sample_x, sample_y);
            let _pixel_value = distance_to_pixel(sampled_distance);
        }
    }
}

pub fn sdf_sample(sdf: &fonts::GlyphEntry, x: f32, y: f32) -> f32 {
    let gx = (x * (sdf.metrics.width as f32) - 0.5).max(0.0);
    let gy = (y * (sdf.metrics.height as f32) - 0.5).max(0.0);
    let left = gx.floor() as usize;
    let top = gy.floor() as usize;
    let wx = gx - (left as f32);
    let wy = gy - (top as f32);

    let right = (left + 1).min((sdf.metrics.width - 1) as usize);
    let bottom = (top + 1).min((sdf.metrics.height - 1) as usize);

    let row_size = sdf.metrics.width as usize;
    let get_pixel = |x, y| sdf.buffer[(row_size * y) + x];

    let p00 = get_pixel(left, top);
    let p10 = get_pixel(right, top);
    let p01 = get_pixel(left, bottom);
    let p11 = get_pixel(right, bottom);

    mix(
        mix(p00 as f32 / 255.0, p10 as f32 / 255.0, wx),
        mix(p01 as f32 / 255.0, p11 as f32 / 255.0, wx),
        wy,
    )
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x-edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn mix(v1: f32, v2: f32, weight: f32) -> f32 {
    v1 + (v2 - v1) * weight
}
