use quote::quote;

use crate::ast::*;
use crate::attr::ParseType;
use crate::parsing_context::*;

pub fn impl_derive(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    let cx = ParsingContext::new();
    let container = match Container::from_ast(&cx, &input) {
        Some(container) => container,
        None => return Err(cx.check().unwrap_err()),
    };
    cx.check()?;

    let ident = &container.ident;
    let result = match &container.data {
        Data::Enum(variants) => {
            let body = serialize_enum(&container, variants);
            quote! {
                impl ton_token_parser::ParseToken<#ident> for ton_abi::TokenValue {
                    fn try_parse(self) -> ton_token_parser::ContractResult<#ident> {
                        #body
                    }
                }

            }
        }
        Data::Struct(_, fields) if container.attrs.plain => {
            let body = serialize_struct(&container, fields, StructType::Plain);
            quote! {
                impl ton_token_parser::ParseToken<#ident> for Vec<ton_abi::Token> {
                    fn try_parse(self) -> ton_token_parser::ContractResult<#ident> {
                        #body
                    }
                }
            }
        }
        Data::Struct(_, fields) => {
            let body = serialize_struct(&container, fields, StructType::Tuple);
            quote! {
                impl ton_token_parser::ParseToken<#ident> for ton_abi::TokenValue {
                    fn try_parse(self) -> ton_token_parser::ContractResult<#ident> {
                        #body
                    }
                }
            }
        }
    };
    Ok(result)
}

fn serialize_enum(_container: &Container, variants: &[Variant]) -> proc_macro2::TokenStream {
    let build_variants = variants
        .iter()
        .filter_map(|variant| {
            variant
                .original
                .discriminant
                .as_ref()
                .map(|(_, discriminant)| (variant.ident.clone(), discriminant))
        })
        .map(|(ident, discriminant)| {
            let token = quote::ToTokens::to_token_stream(discriminant).to_string();
            let number = token.parse::<u8>().unwrap();

            quote! {
                Some(#number) => Ok(EventType::#ident)
            }
        });

    quote! {
        match self {
            ton_abi::TokenValue::Uint(int) => match int.number.to_u8() {
                #(#build_variants,)*
                _ => Err(ton_token_parser::ParserError::InvalidAbi),
            },
            _ => Err(ton_token_parser::ParserError::InvalidAbi),
        }
    }
}

enum StructType {
    Tuple,
    Plain,
}

fn serialize_struct(
    container: &Container,
    fields: &[Field],
    struct_type: StructType,
) -> proc_macro2::TokenStream {
    let name = &container.ident;

    let build_fields = fields.iter().map(|f| {
        let name = f.original.ident.as_ref().unwrap();
        if f.original.attrs.len() == 0 {
            quote! {
               #name: std::default::Default::default()
            }
        } else {
            let field_name = match &f.attrs.name {
                Some(v) => v.clone(),
                None => name.to_string(),
            };
            let parse_type = &f.attrs.parse_type;
            let try_parse = try_parse_struct(parse_type);
            quote! {
                #name: {
                    let token = tokens.next();
                    let name = match &token {
                        Some(token) => token.name.clone(),
                        None => return Err(ton_token_parser::ParserError::InvalidAbi),
                    };
                    if name == #field_name {
                        #try_parse
                    } else {
                        return Err(ton_token_parser::ParserError::InvalidAbi);
                    }
                }
            }
        }
    });

    match struct_type {
        StructType::Plain => {
            quote! {
                let mut tokens = self.into_iter();

                std::result::Result::Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
        StructType::Tuple => {
            quote! {
                let mut tokens = match self {
                    ton_abi::TokenValue::Tuple(tokens) => tokens.into_iter(),
                    _ => return Err(ton_token_parser::ParserError::InvalidAbi),
                };

                std::result::Result::Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
    }
}

fn try_parse_struct(parse_type: &Option<ParseType>) -> proc_macro2::TokenStream {
    match parse_type {
        Some(parse_type) => {
            let handler = get_handler_struct(parse_type);
            quote! {
                match token {
                    Some(token) => {
                        match token.value {
                            #handler
                            _ => return Err(ton_token_parser::ParserError::InvalidAbi),
                        }
                    },
                    None => return Err(ton_token_parser::ParserError::InvalidAbi),
                }
            }
        }
        None => {
            quote! {
                token.try_parse()?
            }
        }
    }
}

fn get_handler_struct(parse_type: &ParseType) -> proc_macro2::TokenStream {
    match parse_type {
        ParseType::UINT8 => {
            quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size:8 }) => {
                    value.to_u8().ok_or(ton_token_parser::ParserError::InvalidAbi)?
                },
            }
        }
        ParseType::UINT16 => {
            quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 16 }) => {
                    value.to_u16().ok_or(ton_token_parser::ParserError::InvalidAbi)?
                },
            }
        }
        ParseType::UINT32 => {
            quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 32 }) => {
                    value.to_u32().ok_or(ton_token_parser::ParserError::InvalidAbi)?
                },
            }
        }
        ParseType::UINT64 => {
            quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 64 }) => {
                    value.to_u64().ok_or(ton_token_parser::ParserError::InvalidAbi)?
                },
            }
        }
        ParseType::UINT128 => {
            quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 128 }) => {
                    value.to_bytes_be().into()
                },
            }
        }
        ParseType::UINT256 => {
            quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 256 }) => {
                    let mut result = [0; 32];
                    let data = value.to_bytes_be();

                    let len = std::cmp::min(data.len(), 32);
                    let offset = 32 - len;
                    (0..len).for_each(|i| result[i + offset] = data[i]);

                    result.into()
                },
            }
        }
        ParseType::ADDRESS => {
            quote! {
                TokenValue::Address(ton_block::MsgAddress::AddrStd(addr)) => {
                    ton_block::MsgAddressInt::AddrStd(addr)
                },
                TokenValue::Address(ton_block::MsgAddress::AddrVar(addr)) => {
                    ton_block::MsgAddressInt::AddrVar(addr)
                },
            }
        }
        ParseType::BOOL => {
            quote! {
                ton_abi::TokenValue::Bool(value) => value,
            }
        }
        ParseType::NONE => unreachable!(),
    }
}
