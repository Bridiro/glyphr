use minijinja::{Environment, context, Value};

use crate::config::ToFontLoaded;
use crate::generator::generate_font;

/// Filter used by minijinja to escape characters that generates error if direcly placed inside
/// apostrophes (e.g. `'`, `\`, `\n`...).
fn rust_char_escape(value: Value) -> Result<String, minijinja::Error> {
    let s = value.as_str().ok_or_else(|| {
        minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, "expected string")
    })?;
    
    if s.chars().count() != 1 {
        return Err(minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation, 
            "expected single character"
        ));
    }
    
    let ch = s.chars().next().unwrap();
    let escaped = match ch {
        '\'' => "\\'".to_string(),
        '\\' => "\\\\".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        '\0' => "\\0".to_string(),
        c if c.is_control() => format!("\\u{{{:04x}}}", c as u32),
        c => c.to_string(),
    };
    
    Ok(escaped)
}

/// Generates a String containing all the code to write out the macro.
pub fn render<T: ToFontLoaded>(font_config: T) -> String {
    let loaded_fonts = font_config.to_font_loaded();

    let mut env = Environment::new();
    env.add_filter("rust_char_escape", rust_char_escape);
    env.add_template("fonts", include_str!("../templates/fonts.rs.j2"))
        .unwrap();

    let mut output = String::new();
    for loaded_font in &loaded_fonts {
        let mut glyphs = vec![];

        let entries = generate_font(&loaded_font);

        for (entry, character) in entries
            .iter()
            .zip(loaded_font.char_range.iter())
        {
            glyphs.push(context! {
                character => character,
                codepoint => entry.1.name.clone(),
                bitmap_len => entry.0.len(),
                bitmap => entry.0.clone(),
                xmin => entry.1.xmin,
                ymin => entry.1.ymin,
                width => entry.1.width,
                height => entry.1.height,
                advance_width => entry.1.advance_width,
            });
        }

        // Now render fonts.rs with everything
        output.push_str(&env.get_template("fonts")
            .unwrap()
            .render(context! {
                font => context! {
                    name => loaded_font.name,
                    size => loaded_font.px,
                    ascent => loaded_font.font.get_ascent(loaded_font.px as f32),
                    descent => loaded_font.font.get_descent(loaded_font.px as f32),
                    format => loaded_font.format.to_string(),
                    glyphs => glyphs,
                },
            })
            .unwrap());
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use minijinja::value::Value;

    fn val(s: &str) -> Value {
        Value::from(s.to_string())
    }

    #[test]
    fn test_normal_char() {
        assert_eq!(rust_char_escape(val("a")).unwrap(), "a");
        assert_eq!(rust_char_escape(val("Z")).unwrap(), "Z");
        assert_eq!(rust_char_escape(val("7")).unwrap(), "7");
    }

    #[test]
    fn test_escape_chars() {
        assert_eq!(rust_char_escape(val("'")).unwrap(), "\\'");
        assert_eq!(rust_char_escape(val("\\")).unwrap(), "\\\\");
        assert_eq!(rust_char_escape(val("\n")).unwrap(), "\\n");
        assert_eq!(rust_char_escape(val("\r")).unwrap(), "\\r");
        assert_eq!(rust_char_escape(val("\t")).unwrap(), "\\t");
        assert_eq!(rust_char_escape(val("\0")).unwrap(), "\\0");
    }

    #[test]
    fn test_control_char() {
        // ASCII 0x01 (Start of Header)
        assert_eq!(rust_char_escape(Value::from("\u{01}")).unwrap(), "\\u{0001}");
    }

    #[test]
    fn test_unicode_non_escape() {
        assert_eq!(rust_char_escape(Value::from("λ")).unwrap(), "λ");
        assert_eq!(rust_char_escape(Value::from("🦀")).unwrap(), "🦀");
    }

    #[test]
    fn test_invalid_length() {
        let err = rust_char_escape(val("")).unwrap_err();
        assert!(format!("{}", err).contains("expected single character"));

        let err = rust_char_escape(val("ab")).unwrap_err();
        assert!(format!("{}", err).contains("expected single character"));
    }

    #[test]
    fn test_non_string_input() {
        let err = rust_char_escape(Value::from(42)).unwrap_err();
        assert!(format!("{}", err).contains("expected string"));
    }
}
