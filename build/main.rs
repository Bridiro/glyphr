mod config;
mod generator;
mod renderer;

use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let rendered = if std::env::var("DOCS_RS").is_ok() {
        fs::read_to_string("build/fallback_font.rs").expect("Should have been able to read the file")
    } else {
        let loaded_fonts = config::get_config();
        renderer::render(loaded_fonts)
    };
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("fonts.rs"),
        rendered,
    )
    .unwrap();

    Ok(())
}
