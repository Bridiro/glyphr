# Glyphr

This is the successor (only spiritually) of [libraster-sw](https://github.com/eagletrt/libraster-sw). I wrote that library because all the alternatives were completely bloated
and had too much features that I did not use. I just wanted it to be as fast as possible, while possibly maintaining an easy use.

## Features
- Completely intuitive
- You decide how pixel are written on the screen
- No heap allocation
- Compile time font bitmaps generation

## Roadmap
- [x] Loading fonts.json from project root
- [x] Generating what's specified in fonts.json
- [x] Saving the bitmaps somewhere
- [x] Generating the file(s) that the library will use
- [x] Main functionalities, like rasterization...
- [ ] Clear and useful APIs
- [ ] General optimization and refactor (I already know I'll need it)

## How To Build

In the project root, create a `fonts` folder, then inside create a `fonts.json`. The library expect an array of fonts, with some parameters. Here is an example:
```json
[
    {
        "name": "Poppins",
        "path": "Poppins-Regular.ttf",
        "px": 36.0,
        "padding": 1,
        "spread": 15.0,
        "char_range": [33, 126]
    }
]
```
It is kind of straightforward to use, but I'll exaplain it to you:
- `name`: a user-defined name that will be used to choose at runtime which font to use (should be UpperCamelCase as it's used as enum entry)
- `path`: the path of the ttf file (relative to `fonts.json` folder)
- `px`: size in pixel of the font
- `padding`: space in pixel to leave between the glyph and the bitmap borders (0 creates a weird visual artifact, so 1 is better)
- `spread`: distance in pixel that the SDF extends from the edges of each glyph. Generally the lower the number, the higher space will be occupied, but the best upscaled resolution you will have.
- `char_range`: a `u8` array of 2 elements, which defines which characters to generate

After creating this file, and placing the ttfs where you prefer, you can just build and the bitmaps will be created. For now there is no API's to use it so it's just generation. The generated files will only be useful inside the library itself.

> [!NOTE]
> You can also define an enviroment variable called `FONTS_DIR`, that contains the path to `fonts.json` folder (and it's relative to the fonts path inside it)

## How To Use

Firstly you need to define the callback used to write a pixel. The signature must be `fn(u32, u32, u32, &[u32])`.
Then you create the struct `Glyphr`:
```rust
use glyphr::{ Glyphr, fonts::Font };

let mut glyphr_struct = Glyphr::new(
    pixel_callback,
    &mut buffer,  // &[u32]
    buffer_width,
    buffer_height,
    SdfConfig {
        color: 0xffffff,
        scale: 2.1,
        smoothing: 0.4,
        mid_value: 0.5,         // should always be 0.5 except for some edge cases
        font: Font::default(),  // will pick the first one generated
    },
);
```
and to render anything you just call:
```rust
glyphr_struct.render("Hello, World!", 100, 50);
```

This will rasterize the font in the buffer, so you just need to display it.

> [!WARNING]
> Expect the APIs to change.
