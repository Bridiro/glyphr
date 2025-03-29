# Glyphr

This is the successor (only spiritually) of [libraster-sw](https://github.com/eagletrt/libraster-sw). I wrote that library because all the alternatives were completely bloated
and had too much features that I did not use. I just wanted it to be as fast as possible, while possibly maintaining an easy use.

## Features
- As many fonts as you want
- Completely intuitive
- You decide how pixel are written on the screen
- No heap allocation
- Compile time font bitmaps generation

## Roadmap
- [x] Loading fonts.json from project root
- [x] Generating what's specified in fonts.json
- [x] Saving the bitmaps somewhere
- [ ] Generating the file(s) that the library will use
- [ ] Main functionalities, like rasterization...
- [ ] Clear and useful APIs
- [ ] General optimization and refactor (I already know I'll need it)

## How To

First of all, in the project root, create a `fonts` folder, then inside create a `fonts.json`. The library expect an array of fonts, with some parameters. Here is an example:
```json
[
    {
        "name": "Konexy",
        "path": "fonts/KonexyFont.ttf",
        "px": 120.0,
        "padding": 1,
        "spread": 20.0,
        "char_range": [33, 126]
    }
]
```
It is kind of straightforward to use, but I'll exaplain it to you:
- `name`: a user-defined name that will be used to choose at runtime which font to use
- `path`: the path of the ttf file (relative to `Cargo.toml`)
- `px`: size in pixel of the font
- `padding`: space in pixel to leave between the glyph and the bitmap borders
- `spread`: distance in pixel that the SDF extends from the edges of each glyph
- `char_range`: a `u8` array of 2 elements, which defines which characters to generate

After creating this file, and placing the ttfs where you prefer, you can just build and the bitmaps will be created. For now there is no API's to use it so it's just generation. The generated files will only be useful inside the library itself.

