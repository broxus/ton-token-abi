use quote::quote;

use crate::ast::*;
use crate::attr::TypeName;
use crate::parsing_context::*;
use proc_macro2::Ident;

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
            let body = get_enum_body(&container, variants);
            quote! {
                impl ton_token_unpacker::UnpackToken<#ident> for ton_abi::TokenValue {
                    fn unpack(self) -> ton_token_unpacker::ContractResult<#ident> {
                        #body
                    }
                }

            }
        }
        Data::Struct(_, fields) => {
            let body_plain = get_packer_body(&container, fields, StructType::Plain);
            let body_tuple = get_packer_body(&container, fields, StructType::Tuple);
            let token_packer = quote! {
                impl ton_token_packer::PackTokens for #ident {
                    fn pack(self) -> Vec<ton_abi::Token> {
                        #body_plain
                    }
                }

                impl ton_token_packer::BuildToken for #ident {
                    fn token(self, name: &str) -> ton_abi::Token {
                        #body_tuple
                    }
                }
            };

            if container.attrs.plain {
                let body = get_unpacker_body(&container, fields, StructType::Plain);
                quote! {
                    impl ton_token_unpacker::UnpackToken<#ident> for Vec<ton_abi::Token> {
                        fn unpack(self) -> ton_token_unpacker::ContractResult<#ident> {
                            #body
                        }
                    }

                    #token_packer
                }
            } else {
                let body = get_unpacker_body(&container, fields, StructType::Tuple);
                quote! {
                    impl ton_token_unpacker::UnpackToken<#ident> for ton_abi::TokenValue {
                        fn unpack(self) -> ton_token_unpacker::ContractResult<#ident> {
                            #body
                        }
                    }

                    #token_packer
                }
            }
        }
    };
    Ok(result)
}

fn get_enum_body(_container: &Container, variants: &[Variant]) -> proc_macro2::TokenStream {
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
                _ => Err(ton_token_unpacker::UnpackerError::InvalidAbi),
            },
            _ => Err(ton_token_unpacker::UnpackerError::InvalidAbi),
        }
    }
}

fn get_packer_body(
    _container: &Container,
    fields: &[Field],
    struct_type: StructType,
) -> proc_macro2::TokenStream {
    let definition = quote! {
        let mut tokens: Vec<ton_abi::Token> = Vec::new();
    };

    let build_fields = fields.iter().map(|f| {
        if f.original.attrs.is_empty() {
            quote! {} // do nothing
        } else {
            let name = f.original.ident.as_ref().unwrap();
            let field_name = match &f.attrs.name {
                Some(v) => v.clone(),
                None => name.to_string(),
            };

            match &f.attrs.pack_with {
                Some(data) => {
                    quote! {
                        tokens.push(#data(self.#name, #field_name))
                    }
                }
                None => match &f.attrs.type_name {
                    Some(type_name) => {
                        let handler = get_handler(type_name, MethodType::Packer(name));
                        quote! {
                            tokens.push(ton_abi::Token::new(#field_name, #handler))
                        }
                    }
                    None => {
                        quote! {
                            tokens.push(self.#name.token(#field_name))
                        }
                    }
                },
            }
        }
    });

    match struct_type {
        StructType::Plain => {
            quote! {
                #definition
                #(#build_fields;)*
                return tokens;
            }
        }
        StructType::Tuple => {
            quote! {
                #definition
                #(#build_fields;)*
                return tokens.token(name);
            }
        }
    }
}

fn get_unpacker_body(
    container: &Container,
    fields: &[Field],
    struct_type: StructType,
) -> proc_macro2::TokenStream {
    let name = &container.ident;

    let build_fields = fields.iter().map(|f| {
        let name = f.original.ident.as_ref().unwrap();
        if f.original.attrs.is_empty() {
            quote! {
               #name: std::default::Default::default()
            }
        } else {
            let field_name = match &f.attrs.name {
                Some(v) => v.clone(),
                None => name.to_string(),
            };

            let try_unpack = try_unpack(&f.attrs.type_name, &f.attrs.unpack_with);

            quote! {
                #name: {
                    let token = tokens.next();
                    let name = match &token {
                        Some(token) => token.name.clone(),
                        None => return Err(ton_token_unpacker::UnpackerError::InvalidAbi),
                    };
                    if name == #field_name {
                        #try_unpack
                    } else {
                        return Err(ton_token_unpacker::UnpackerError::InvalidAbi);
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
                    _ => return Err(ton_token_unpacker::UnpackerError::InvalidAbi),
                };

                std::result::Result::Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
    }
}

fn try_unpack(
    type_name: &Option<TypeName>,
    unpack_with: &Option<syn::Expr>,
) -> proc_macro2::TokenStream {
    match unpack_with {
        Some(data) => quote! {
            match token {
                Some(token) => #data(&token.value)?,
                None => return Err(ton_token_unpacker::UnpackerError::InvalidAbi),
            }
        },
        None => match type_name {
            Some(type_name) => {
                let handler = get_handler(type_name, MethodType::Unpacker);
                quote! {
                    match token {
                        Some(token) => {
                            match token.value {
                                #handler
                                _ => return Err(ton_token_unpacker::UnpackerError::InvalidAbi),
                            }
                        },
                        None => return Err(ton_token_unpacker::UnpackerError::InvalidAbi),
                    }
                }
            }
            None => {
                quote! {
                    token.unpack()?
                }
            }
        },
    }
}

fn get_handler(type_name: &TypeName, method_type: MethodType) -> proc_macro2::TokenStream {
    match type_name {
        TypeName::Int(size) => {
            if *size <= 8 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: ton_token_unpacker::bigint::BigInt::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: value, size: #size }) => {
                                value.to_i8().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 8 && *size <= 16 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: ton_token_unpacker::bigint::BigInt::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: value, size: #size }) => {
                                value.to_i16().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 16 && *size <= 32 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: ton_token_unpacker::bigint::BigInt::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: value, size: #size }) => {
                                value.to_i32().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 32 && *size <= 64 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: ton_token_unpacker::bigint::BigInt::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: value, size: #size }) => {
                                value.to_i64().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 64 && *size <= 128 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: ton_token_unpacker::bigint::BigInt::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Int(ton_abi::Int { number: value, size: #size }) => {
                                value.to_i128().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else {
                unreachable!()
            }
        }
        TypeName::Uint(size) => {
            if *size <= 8 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: ton_token_unpacker::bigint::BigUint::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: #size }) => {
                                value.to_u8().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 8 && *size <= 16 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: ton_token_unpacker::bigint::BigUint::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: #size }) => {
                                value.to_u16().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 16 && *size <= 32 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: ton_token_unpacker::bigint::BigUint::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: #size }) => {
                                value.to_u32().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 32 && *size <= 64 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: ton_token_unpacker::bigint::BigUint::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: #size }) => {
                                value.to_u64().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 64 && *size <= 128 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: ton_token_unpacker::bigint::BigUint::from(self.#name), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: #size }) => {
                                value.to_u128().ok_or(ton_token_unpacker::UnpackerError::InvalidAbi)?
                            },
                        }
                    }
                }
            } else if *size > 128 && *size <= 256 {
                match method_type {
                    MethodType::Packer(name) => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: ton_token_unpacker::bigint::BigUint::from_bytes_be(self.#name.as_slice()), size: #size })
                        }
                    }
                    MethodType::Unpacker => {
                        quote! {
                            ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: #size }) => {
                                let mut result = [0; 32];
                                let data = value.to_bytes_be();

                                let len = std::cmp::min(data.len(), 32);
                                let offset = 32 - len;
                                (0..len).for_each(|i| result[i + offset] = data[i]);

                                result.into()
                            },
                        }
                    }
                }
            } else {
                unreachable!()
            }
        }
        TypeName::Address => match method_type {
            MethodType::Packer(name) => {
                quote! {
                    ton_abi::TokenValue::Address(match self.#name {
                        ton_block::MsgAddressInt::AddrStd(addr) => ton_block::MsgAddress::AddrStd(addr),
                        ton_block::MsgAddressInt::AddrVar(addr) => ton_block::MsgAddress::AddrVar(addr),
                    })
                }
            }
            MethodType::Unpacker => {
                quote! {
                    ton_abi::TokenValue::Address(ton_block::MsgAddress::AddrStd(addr)) => {
                        ton_block::MsgAddressInt::AddrStd(addr)
                    },
                    ton_abi::TokenValue::Address(ton_block::MsgAddress::AddrVar(addr)) => {
                        ton_block::MsgAddressInt::AddrVar(addr)
                    },
                }
            }
        },
        TypeName::Cell => match method_type {
            MethodType::Packer(name) => {
                quote! {
                    ton_abi::TokenValue::Cell(self.#name)
                }
            }
            MethodType::Unpacker => {
                quote! {
                    ton_abi::TokenValue::Cell(cell) => cell,
                }
            }
        },
        TypeName::Bool => match method_type {
            MethodType::Packer(name) => {
                quote! {
                    ton_abi::TokenValue::Bool(self.#name)
                }
            }
            MethodType::Unpacker => {
                quote! {
                    ton_abi::TokenValue::Bool(value) => value,
                }
            }
        },
        TypeName::None => unreachable!(),
    }
}

enum StructType {
    Tuple,
    Plain,
}

enum MethodType<'a> {
    Packer(&'a Ident),
    Unpacker,
}
