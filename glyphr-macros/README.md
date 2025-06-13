# glyphr-macros

This crate contains proc-macros used to generate code at compile time easily.

## generate_font!

```rust
glyphr_macros::generate_font! {
    name: POPPINS,
    path: "fonts/Poppins-Regular.ttf",
    size: 64,
    characters: "A-Za-z0-9 !$£%&",
    format: SDF {
        spread: 20.0,
        padding: 0,
    },
}
```

## generate_font_from_toml!

```rust
glyphr_macros::generate_font_from_toml!("fonts/fonts.toml");
```
with the toml looking like this:
```toml
[[font]]
name = "POPPINS"
path = "Poppins-Regular.ttf"
size = 64
characters = "A-Za-z0-9 !$£%&"
format = { SDF = { spread = 20.0, padding = 0}}

```

format can either be `SDF` or `Bitmap`.


## Differences between the 2 formats:

### SDF

SDF is the prettier format, as it enables the user to resize the generated code at runtime.
As resizing and calculating positions require computation, it's suggested to use this option for text that needs to be pretty,
but at the same time not with the fastest refresh rate (only if you're on an MCU or similar).

### Bitmap

This is the fast and memory efficient way. Font generated this way can't be rescaled at runtime, but it's generally **8x** smaller
than an SDF generated font. This means that if you only need from 1 to 4 sizes, you can use this to save space.
