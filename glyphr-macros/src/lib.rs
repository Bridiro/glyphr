mod config;
mod generator;
mod macro_parser;
mod renderer;

#[cfg(feature = "toml")]
mod toml_parser;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use macro_parser::FontConfig;

/// Macro used to generate a font with data direcly in the code
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

/// The underlying process is the same as `generate_font!` macro, but can do more at the same time by
/// specifing fonts in a `toml` file as specified in `README.md`
#[cfg(feature = "toml")]
#[proc_macro]
pub fn generate_fonts_from_toml(input: TokenStream) -> TokenStream {
    use std::fs;
    use syn::LitStr;
    use toml;

    let file_path = parse_macro_input!(input as LitStr);
    let path_str = file_path.value();
    let content = match fs::read_to_string(&path_str) {
        Ok(content) => content,
        Err(err) => {
            return syn::Error::new_spanned(
                file_path,
                format!("Failed to read file '{}': {}", path_str, err),
            )
            .to_compile_error()
            .into();
        }
    };
    let mut toml_input: toml_parser::TomlConfig =
        toml::from_str(&content).expect("Could not parse toml file");
    toml_input.relativize_paths(&path_str);

    let rendered = renderer::render(toml_input);

    match rendered.parse() {
        Ok(parsed) => parsed,
        Err(e) => {
            panic!("Failed to generate font: {}", e)
        }
    }
}
