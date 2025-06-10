mod config;
mod generator;
mod renderer;
mod macro_parser;

use proc_macro::TokenStream;
use syn::{parse_macro_input};
use macro_parser::FontConfig;

#[proc_macro]
pub fn generate_font(input: TokenStream) -> TokenStream {
    let font_input: FontConfig = parse_macro_input!(input as FontConfig);

    let rendered = renderer::render(font_input);

    match rendered.parse() {
        Ok(parsed) => parsed,
        Err(e) => {
            panic!("Failed to generate font: {}", e)
        }
    }
}

