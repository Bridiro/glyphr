use crate::config::BitmapFormat;
use syn::{Error, Ident, LitFloat, LitInt, LitStr, Token, parse::Parse};

pub struct FontConfig {
    pub name: Ident,
    pub path: String,
    pub size: i32,
    pub characters: String,
    pub format: BitmapFormat,
}

impl Parse for FontConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut path = None;
        let mut size = None;
        let mut characters = None;
        let mut format = None;

        while !input.is_empty() {
            let field_name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match field_name.to_string().as_str() {
                "name" => {
                    name = Some(input.parse::<Ident>()?);
                }
                "path" => {
                    path = Some(input.parse::<LitStr>()?.value());
                }
                "size" => {
                    size = Some(input.parse::<LitInt>()?.base10_parse::<i32>()?);
                }
                "characters" => {
                    characters = Some(input.parse::<LitStr>()?.value());
                }
                "format" => {
                    format = Some(parse_format(input)?);
                }
                _ => {
                    return Err(Error::new(field_name.span(), "Unknown field"));
                }
            }

            // Optional comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(FontConfig {
            name: name.ok_or_else(|| Error::new(input.span(), "Missing 'name' field"))?,
            path: path.ok_or_else(|| Error::new(input.span(), "Missing 'path' field"))?,
            size: size.ok_or_else(|| Error::new(input.span(), "Missing 'size' field"))?,
            characters: characters
                .ok_or_else(|| Error::new(input.span(), "Missing 'characters' field"))?,
            format: format.ok_or_else(|| Error::new(input.span(), "Missing 'format' field"))?,
        })
    }
}

fn parse_format(input: syn::parse::ParseStream) -> syn::Result<BitmapFormat> {
    if input.peek(Ident) {
        let format_name: Ident = input.parse()?;
        let mut spread = None;
        let mut padding = None;

        let content;
        syn::braced!(content in input);
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "spread" => {
                    spread = Some(content.parse::<LitFloat>()?.base10_parse::<f32>()?);
                }
                "padding" => {
                    padding = Some(content.parse::<LitInt>()?.base10_parse::<i32>()?);
                }
                _ => {
                    return Err(syn::Error::new(key.span(), "Unknown Format field"));
                }
            }
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        match format_name.to_string().as_str() {
            "Bitmap" => Ok(BitmapFormat::Bitmap {
                spread: spread
                    .ok_or_else(|| Error::new(content.span(), "Missing 'spread' field"))?,
                padding: padding
                    .ok_or_else(|| Error::new(content.span(), "Missing 'padding' field"))?,
            }),
            "SDF" => Ok(BitmapFormat::SDF {
                spread: spread
                    .ok_or_else(|| Error::new(content.span(), "Missing 'spread' field"))?,
                padding: padding
                    .ok_or_else(|| Error::new(content.span(), "Missing 'padding' field"))?,
            }),
            _ => Err(Error::new(format_name.span(), "Unknown format")),
        }
    } else {
        Err(Error::new(input.span(), "No format name provided"))
    }
}
