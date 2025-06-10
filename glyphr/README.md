# Glyphr

[![License](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/Bridiro/glyphr#license)
[![Crates.io](https://img.shields.io/crates/v/glyphr.svg)](https://crates.io/crates/glyphr)
[![Downloads](https://img.shields.io/crates/d/glyphr.svg)](https://crates.io/crates/glyphr)
[![Docs](https://docs.rs/glyphr/badge.svg)](https://docs.rs/glyphr/latest/glyphr/)
[![CI](https://github.com/Bridiro/glyphr/actions/workflows/rust.yml/badge.svg)](https://github.com/Bridiro/glyphr/actions)

This is the successor (only spiritually) of [libraster-sw](https://github.com/eagletrt/libraster-sw). I wrote that library because all the alternatives were completely bloated
and had too much features that I did not use. I just wanted it to be as fast as possible, while possibly maintaining an easy use.

## Features
- Completely intuitive
- You decide how pixel are written on the screen
- No heap allocation
- Compile time font bitmaps generation
- Full Unicode support

## How To Build

In the project root, create a `fonts` folder, then inside create a `fonts.toml`. The library expect an array of fonts, with some parameters. Here is an example:
```toml
[[font]]
name = "Poppins"
path = "Poppins-Regular.ttf"
px = 64.0
padding = 1
spread = 20.0
char_range = "A-Za-z0-9& "
```
It is kind of straightforward to use, but I'll exaplain it to you:
- `name`: a user-defined name that will be used to choose at runtime which font to use (should be UpperCamelCase as it's used as enum entry)
- `path`: the path of the ttf file (relative to `fonts.toml` folder)
- `px`: size in pixel of the font
- `padding`: space in pixel to leave between the glyph and the bitmap borders (0 creates a weird visual artifact, so 1 is better)
- `spread`: distance in pixel that the SDF extends from the edges of each glyph. Generally the lower the number, the higher space will be occupied, but the best upscaled resolution you will have.
- `char_range`: a `String` regex-like used to define which characters to generate.

After creating this file, and placing the ttfs where you prefer, you can just build and the bitmaps will be created. For now there is no API's to use it so it's just generation. The generated files will only be useful inside the library itself.

> [!NOTE]
> You can also define an enviroment variable called `GLYPHR_CONFIG`, that contains the path to `fonts.toml` folder (and it's relative to the fonts path inside it)

## How To Use

Firstly you need to define the callback used to write a pixel. The signature must be `fn(u32, u32, u32, &[u32])`.
Then you create the struct `Glyphr`:
```rust
use glyphr::{ Glyphr, fonts::{ Font, FontAlign } };

let mut glyphr_struct = Glyphr::new(
    pixel_callback,
    &mut buffer,  // &[u32]
    buffer_width,
    buffer_height,
    SdfConfig {
        color: 0xffffff,
        px: 70,
        smoothing: 0.4,
        mid_value: 0.5,              // should always be 0.5 except for some edge cases
    },
);
```
and to render anything you just call:
```rust
glyphr_struct.render("Hello, World!", 100, 50);
```

> [!TIP]
> If you want to run an example on your machine you can just do:
> ```rust
> cargo run --example glyphr_test_window --features window
> ```

This will rasterize the font in the buffer, so you just need to display it.

> [!WARNING]
> Expect the APIs to change.
