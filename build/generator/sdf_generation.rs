use crate::generator::line;

pub struct SdfRaster {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<f32>,
}

pub fn sdf_generate(
    width: u32,
    height: u32,
    padding: i32,
    spread: f32,
    lines: &[line::Line],
) -> SdfRaster {
    let mut lines = lines;
    let mut padded_lines: Vec<line::Line> = Vec::with_capacity(lines.len());
    if padding != 0 {
        let padding_width_normalized = padding as f32 / width as f32;
        let padding_height_normalized = padding as f32 / height as f32;
        for line in lines.iter() {
            padded_lines.push(line.normalize_to_with_offset(
                -padding_width_normalized,
                -padding_height_normalized,
                1.0 as f32 + (padding_width_normalized * 2.0),
                1.0 as f32 + (padding_height_normalized * 2.0),
            ));
        }

        lines = padded_lines.as_slice();
    }

    let _1w = 1.0 / width as f32;
    let _1h = 1.0 / height as f32;

    let buffer_size = (width * height) as usize;
    let mut image_buffer: Vec<f32> = vec![0.0; buffer_size];

    for x in 0..width {
        for y in 0..height {
            let px = (x as f32 + 0.5) * _1w;
            let py = (y as f32 + 0.5) * _1h;
            let index = (x + (width * y)) as usize;

            let mut min_distance = f32::MAX;
            for line in lines {
                let d = line.distance(px, py);
                if d < min_distance {
                    min_distance = d;
                }
            }

            min_distance = (1.0 - (min_distance * spread)) - 0.5;
            image_buffer[index] = min_distance.clamp(0.0, 1.0);
        }
    }

    for y in 0..height {
        let py = (y as f32 + 0.5) * _1h;
        let scanline = scanline(py, lines);

        for x in 0..width {
            let index = (x + (width * y)) as usize;
            let px = (x as f32 + 0.5) * _1w;

            if scanline_scan(&scanline, px) {
                image_buffer[index] = 1.0 - image_buffer[index];
            }
        }
    }

    SdfRaster {
        width,
        height,
        buffer: image_buffer,
    }
}

pub fn sdf_to_bitmap(sdf: &SdfRaster) -> Vec<u8> {
    let width = sdf.width;
    let height = sdf.height;
    let mut buffer: Vec<u8> = vec![0u8; (width * height) as usize];

    for x in 0..width {
        for y in 0..height {
            let index = (x + (width * y)) as usize;
            buffer[index] = (sdf.buffer[index] * 255.0) as u8;
        }
    }

    buffer
}

struct Scanline {
    intersections: Vec<f32>,
}

fn scanline(y: f32, lines: &[line::Line]) -> Scanline {
    let mut scanline = Scanline {
        intersections: Vec::with_capacity(16),
    };
    let mut x = [0.0, 0.0, 0.0];

    for line in lines {
        let count = line.intersections(y, &mut x);
        for i in 0..count {
            scanline.intersections.push(x[i]);
        }
    }

    if scanline.intersections.len() > 0 {
        scanline
            .intersections
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        scanline.intersections.dedup();
    }

    scanline
}

fn scanline_scan(scanline: &Scanline, x: f32) -> bool {
    let count = scanline
        .intersections
        .iter()
        .fold(0u32, |acc, &inter| match x < inter {
            true => acc + 1,
            false => acc,
        });

    count % 2 == 1
}

pub fn mix(v1: f32, v2: f32, weight: f32) -> f32 {
    v1 + (v2 - v1) * weight
}
