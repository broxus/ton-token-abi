use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TokenAbi, attributes(abi))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };

    let build_fields = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();

        if abi_of(&f).is_some() {
            let (an, at) = get_attributes(&f);
            let attr_name = match an {
                Some(v) => v,
                None => name.to_string(),
            };

            let try_parse = try_parse(at);

            quote! {
                #name: {
                    let token = tokens.next();
                    let name = match &token {
                        Some(token) => token.name.clone(),
                        None => return Err(ton_token_parser::ParserError::InvalidAbi),
                    };
                    if name == #attr_name {
                        #try_parse
                    } else {
                        return Err(ton_token_parser::ParserError::InvalidAbi);
                    }
                }
            }
        } else {
            quote! {
               #name: std::default::Default::default()
            }
        }
    });

    let expanded = quote! {
        impl ton_token_parser::ParseToken<#name> for ton_abi::TokenValue {
            fn try_parse(self) -> ton_token_parser::ContractResult<#name> {
                let mut tokens = match self {
                    ton_abi::TokenValue::Tuple(tokens) => tokens.into_iter(),
                    _ => return Err(ton_token_parser::ParserError::InvalidAbi),
                };

                std::result::Result::Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
    };

    expanded.into()
}

fn abi_of(f: &syn::Field) -> Option<&syn::Attribute> {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "abi" {
            return Some(attr);
        }
    }
    None
}

fn get_attributes(f: &syn::Field) -> (Option<String>, Option<String>) {
    let mut name: Option<String> = None;
    let mut typename: Option<String> = None;

    let g = abi_of(f).unwrap_or_else(|| panic!("Expected `abi(name = \"...\", \"$typename\")`"));

    match g.parse_meta() {
        Ok(syn::Meta::List(nvs)) => {
            // list here is .. in #[abi(..)]
            assert!(nvs.path.is_ident("abi"));
            if nvs.nested.len() > 2 {
                panic!("Expected `abi(name = \"...\", \"$typename\")`");
            }

            for meta in nvs.nested.into_iter() {
                match meta {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                        if nv.path.is_ident("name") {
                            match nv.lit {
                                syn::Lit::Str(s) => {
                                    name = Some(s.value());
                                }
                                lit => panic!("Expected string, found {:?}", lit),
                            }
                        } else {
                            panic!("Expected: `name`; found: {:?}", nv.lit);
                        }
                    }
                    syn::NestedMeta::Lit(l) => match l {
                        syn::Lit::Str(s) => {
                            typename = Some(s.value());
                        }
                        lit => panic!("Expected string, found {:?}", lit),
                    },
                    _ => panic!("Invalid nested attribute format"),
                }
            }
        }
        Ok(syn::Meta::Path(_)) => {
            // inside of #[] there was just an identifier (`#[abi]`)
            // include to parsing without attributes
        }
        Ok(syn::Meta::NameValue(name_value)) => {
            panic!(
                "Inside of #[] there was just key-value mapping (`#[abi = \"...\"]`): {:?}",
                name_value.lit
            );
        }
        Err(err) => {
            panic!("Failed to parse the content of the attribute: {:?}", err);
        }
    }

    (name, typename)
}

fn try_parse(typename: Option<String>) -> proc_macro2::TokenStream {
    match typename {
        Some(typename) => {
            let handler = typename_handler(typename);
            let parser = quote! {
                match token {
                    Some(token) => {
                        match token.value {
                            #handler
                            _ => return Err(ton_token_parser::ParserError::InvalidAbi),
                        }
                    },
                    None => return Err(ton_token_parser::ParserError::InvalidAbi),
                }
            };
            parser
        }
        None => {
            let parser = quote! {
                token.try_parse()?
            };
            parser
        }
    }
}

fn typename_handler(typename: String) -> proc_macro2::TokenStream {
    match typename.as_str() {
        "uint8" => {
            let handler = quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size:8 }) => {
                    value.to_u8().ok_or(ton_token_parser::ParserError::InvalidAbi)?
                },
            };
            handler
        }
        "uint16" => {
            let handler = quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 16 }) => {
                    value.to_u16().ok_or(ton_token_parser::ParserError::InvalidAbi)?
                },
            };
            handler
        }
        "uint32" => {
            let handler = quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 32 }) => {
                    value.to_u32().ok_or(ton_token_parser::ParserError::InvalidAbi)?
                },
            };
            handler
        }
        "uint64" => {
            let handler = quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 64 }) => {
                    value.to_u64().ok_or(ton_token_parser::ParserError::InvalidAbi)?
                },
            };
            handler
        }
        "uint128" => {
            let handler = quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 128 }) => {
                    value.to_bytes_be().into()
                },
            };
            handler
        }
        "uint256" => {
            let handler = quote! {
                ton_abi::TokenValue::Uint(ton_abi::Uint { number: value, size: 256 }) => {
                    let mut result = [0; 32];
                    let data = value.to_bytes_be();

                    let len = std::cmp::min(data.len(), 32);
                    let offset = 32 - len;
                    (0..len).for_each(|i| result[i + offset] = data[i]);

                    result.into()
                },
            };
            handler
        }
        "bool" => {
            let handler = quote! {
                ton_abi::TokenValue::Bool(value) => value,
            };
            handler
        }
        _ => panic!("Wrong type name"),
    }
}
