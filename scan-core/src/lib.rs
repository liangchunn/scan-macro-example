pub use error::{CompileError, ExtractError};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, Expr, LitStr, Token};

mod error;

pub struct MacroInput {
    template: LitStr,
    _comma: Token![,],
    data: Expr,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            template: input.parse()?,
            _comma: input.parse()?,
            data: input.parse()?,
        })
    }
}

pub fn generate(input: MacroInput) -> TokenStream {
    let template_str = input.template.value();
    let template = parse_template(&template_str);
    let data = &input.data;

    match template {
        Ok(template) => quote! {
            (|| -> Result<_, ::scan::ExtractError> {
                let extracted = ::scan::extract(#template_str, #data)?;
                Ok(#template)
            })()
        },
        Err(e) => syn::Error::new(input.template.span(), e).to_compile_error(),
    }
}

impl ToTokens for Template {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let all_tokens = self
            .0
            .iter()
            .enumerate()
            .map(|(index, template_type)| {
                let getter = quote!(
                    let value = extracted.get(#index).ok_or(::scan::ExtractError::InternalMissingValueAtIndex(#index))?
                );

                let converter = match template_type {
                    Placeholder::Byte => quote! {
                        u8::from_be_bytes(<[u8; 1]>::try_from(*value).map_err(::scan::ExtractError::InternalTryError)?)
                    },
                    Placeholder::Word => quote! {
                        u16::from_be_bytes(<[u8; 2]>::try_from(*value).map_err(::scan::ExtractError::InternalTryError)?)
                    },
                    Placeholder::Double => quote! {
                        u32::from_be_bytes(<[u8; 4]>::try_from(*value).map_err(::scan::ExtractError::InternalTryError)?)
                    },
                    Placeholder::Quad => quote! {
                        u64::from_be_bytes(<[u8; 8]>::try_from(*value).map_err(::scan::ExtractError::InternalTryError)?)
                    },
                    Placeholder::RawRest => quote! {
                        *value
                    },
                };

                quote!({
                    #getter;
                    #converter
                })
            })
            .collect::<Vec<_>>();

        tokens.extend(quote!(
            (#(#all_tokens),*)
        ))
    }
}

struct Template(Vec<Placeholder>);

enum Placeholder {
    Byte,
    Word,
    Double,
    Quad,
    RawRest,
}

fn parse_template(input: &str) -> Result<Template, CompileError> {
    let mut buf = Vec::with_capacity(2);
    let mut placeholders: Vec<Placeholder> = vec![];
    let mut mark_last = false;
    for char in input.chars() {
        if buf.len() < 2 {
            if char.is_whitespace() {
                continue;
            }
            if mark_last {
                return Err(CompileError::WildcardNotLast);
            }
            buf.push(char);
        }
        if buf.len() == 2 {
            if buf[1] == '%' {
                return Err(CompileError::InvalidFormatPlacement(buf[0]));
            }
            if buf[0] == '%' {
                let t_type = match buf[1] {
                    'b' => Ok(Placeholder::Byte),
                    'w' => Ok(Placeholder::Word),
                    'd' => Ok(Placeholder::Double),
                    'q' => Ok(Placeholder::Quad),
                    '*' => {
                        mark_last = true;
                        Ok(Placeholder::RawRest)
                    }
                    '?' => {
                        buf.clear();
                        continue;
                    }
                    char => Err(CompileError::InvalidFormatCharacter(char)),
                }?;

                placeholders.push(t_type);
            } else {
                let hex_str = buf.iter().collect::<String>();
                let num = u8::from_str_radix(&hex_str, 16);
                if num.is_err() {
                    return Err(CompileError::InvalidHexNumber(hex_str));
                }
            }
            buf.clear();
        }
    }
    if !buf.is_empty() {
        return Err(CompileError::UnmatchedCharacter(
            buf.iter().collect::<String>(),
        ));
    }
    Ok(Template(placeholders))
}

pub fn extract<'a>(template: &'a str, data: &'a [u8]) -> Result<Vec<&'a [u8]>, ExtractError> {
    let mut index = 0;
    let mut buf = Vec::with_capacity(2);
    let mut result = vec![];
    for char in template.chars() {
        if buf.len() < 2 {
            if char.is_whitespace() {
                continue;
            }
            if !matches!(char, 'a'..='z' | 'A'..='Z' | '0'..='9' | '%' | '?' | '*') {
                return Err(ExtractError::InvalidFormatCharacter(char));
            }
            buf.push(char);
        }
        if buf.len() == 2 {
            if buf[0] == '%' {
                match buf[1] {
                    '?' => {
                        index += 1;
                    }
                    'b' => {
                        let slice = data
                            .get(index..index + 1)
                            .ok_or(ExtractError::UnmatchedByte)?;
                        result.push(slice);
                        index += 1;
                    }
                    'w' => {
                        let slice = data
                            .get(index..index + 2)
                            .ok_or(ExtractError::UnmatchedWord)?;
                        result.push(slice);
                        index += 2;
                    }
                    'd' => {
                        let slice = data
                            .get(index..index + 4)
                            .ok_or(ExtractError::UnmatchedDouble)?;
                        result.push(slice);
                        index += 4;
                    }
                    'q' => {
                        let slice = data
                            .get(index..index + 8)
                            .ok_or(ExtractError::UnmatchedQuad)?;
                        result.push(slice);
                        index += 8;
                    }
                    '*' => {
                        let slice = data
                            .get(index..data.len())
                            .ok_or(ExtractError::UnmatchedRestBytes)?;
                        result.push(slice);
                        index += data.len() - index;
                    }
                    _ => unreachable!(),
                }
            } else {
                let hex_str = buf.iter().collect::<String>();
                let num = u8::from_str_radix(&hex_str, 16)?;
                let compare = data.get(index).ok_or(ExtractError::MissingValue(num))?;
                if num != *compare {
                    return Err(ExtractError::MismatchedValue(*compare, num));
                }

                index += 1;
            }
            // clear the string buffer
            buf.clear()
        }
    }
    if !buf.is_empty() {
        return Err(ExtractError::UnmatchedCharacter(
            buf.iter().collect::<String>(),
        ));
    }
    if index != data.len() {
        return Err(ExtractError::ResidualData);
    }
    Ok(result)
}
