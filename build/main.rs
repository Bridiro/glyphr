mod config;
mod generator;
mod renderer;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var("DOCS_RS").is_ok() {
        return Ok(());
    };

    let loaded_fonts = config::get_config();
    renderer::render(loaded_fonts);
    Ok(())
}
