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

To decide how to write pixels you can use `BufferTarget` (only if you're using a `[u32]` array). If you're using a custom target you need to implement the `RenderTarget` trait on it.
Then you create the struct `Glyphr`:
```rust
use glyphr::{ Glyphr, BufferTarget, RenderConfig, SdfConfig };

let mut target = BufferTarget::new(&mut buffer, 800, 480);
let conf = RenderConfig {
    color: 0xffffff,
    sdf: SdfConfig {
        size: 64,
        mid_value: 0.5,
        smoothing: 0.5,
    }
};
let renderer = Glyphr::with_config(conf);
```
and to render anything you just call:
```rust
use glyphr::{ TextAlign, AlignV, AlignH };

renderer.render(&mut target, "Hello World!", POPPINS, 100, 50, TextAlign { horizontal: AlignH::Left, vertical: AlignV::Baseline }).unwrap();
```

> [!TIP]
> If you want to run an example on your machine you can just do:
> ```rust
> cargo run --example glyphr_test_window --features window
> ```

This will rasterize the font in the buffer, so you just need to display it.

> [!WARNING]
> Expect the APIs to change.
