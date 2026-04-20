# Glyphr

[![License](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/Bridiro/glyphr#license)
[![Crates.io](https://img.shields.io/crates/v/glyphr.svg)](https://crates.io/crates/glyphr)
[![Downloads](https://img.shields.io/crates/d/glyphr.svg)](https://crates.io/crates/glyphr)
[![Docs](https://docs.rs/glyphr/badge.svg)](https://docs.rs/glyphr/latest/glyphr/)
[![CI](https://github.com/Bridiro/glyphr/actions/workflows/rust.yml/badge.svg)](https://github.com/Bridiro/glyphr/actions)

This library focus is not to be the fastest, but one of the most beautiful in the embedded world.

## Features
- Completely intuitive
- You decide how pixel are written on the screen
- No heap allocation
- Compile time font bitmaps generation
- Full Unicode support

## How To Build
To get started visit [glyphr-macros](https://github.com/Bridiro/glyphr/tree/master/glyphr-macros) for detailed instructions on how to generate fonts, then proceed in this page.

## How To Use

The renderer is callback-first. You provide dimensions and closures; `glyphr` does not store your
framebuffer.

Create `Glyphr` with a builder-style config:
```rust
use glyphr::{Callbacks, Glyphr, RenderConfig, SdfConfig};

let conf = RenderConfig::default()
    .with_color(0x00ff_ffff)
    .with_sdf(SdfConfig::default().with_size(64).with_smoothing(0.5));
let renderer = Glyphr::with_config(conf);

let mut target = Callbacks::new(800, 480, |x, y, argb| {
    // Your callback decides how to write pixels.
    let _ = (x, y, argb);
    true
});
```

Render text with `draw_text`:
```rust
use glyphr::{ TextAlign, AlignV, AlignH };

renderer
    .draw_text(
        &mut target,
        "Hello World!",
        POPPINS,
        100,
        50,
        TextAlign::new(AlignH::Left, AlignV::Baseline),
    )
    .unwrap();
```

For DMA-style pipelines, use `draw_text_bulk` to get one ARGB tile per glyph:
```rust
use glyphr::{AlignH, AlignV, BulkCallbacks, TextAlign};

let mut scratch = [0u32; 4096];
let mut bulk = BulkCallbacks::new(800, 480, &mut scratch, |x, y, w, h, pixels| {
    // Example: queue one DMA2D blit for this glyph tile.
    let _ = (x, y, w, h, pixels);
    true
});

renderer
    .draw_text_bulk(
        &mut bulk,
        "Hello DMA2D!",
        POPPINS,
        100,
        50,
        TextAlign::new(AlignH::Left, AlignV::Baseline),
    )
    .unwrap();
```

> [!TIP]
> If you want to run an example on your machine you can just do:
> ```rust
> cargo run --example glyphr_test --features "toml window"
> ```
