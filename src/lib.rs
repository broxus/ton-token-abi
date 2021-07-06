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
            let attr_name = match name_of(&f) {
                Some(v) => v,
                None => name.to_string(),
            };

            quote! {
                #name: {
                    let token = tokens.next();
                    let name = match &token {
                        Some(token) => token.name.clone(),
                        None => return Err(nekoton::helpers::abi::ParserError::InvalidAbi),
                    };
                    if name == #attr_name {
                        token.try_parse()?
                    } else {
                        return Err(nekoton::helpers::abi::ParserError::InvalidAbi);
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
        impl nekoton::helpers::abi::ParseToken<#name> for ton_abi::TokenValue {
            fn try_parse(self) -> nekoton::helpers::abi::ContractResult<#name> {
                let mut tokens = match self {
                    ton_abi::TokenValue::Tuple(tokens) => tokens.into_iter(),
                    _ => return Err(nekoton::helpers::abi::ParserError::InvalidAbi),
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

fn name_of(f: &syn::Field) -> Option<String> {
    let name: Option<String>;

    let g = abi_of(f)?;

    let meta = match g.parse_meta() {
        Ok(syn::Meta::List(mut nvs)) => {
            // list here is .. in #[abi(..)]
            assert!(nvs.path.is_ident("abi"));
            if nvs.nested.len() != 1 {
                panic!("Expected `abi(name = \"...\")`");
            }

            // nvs.nested[0] here is (hopefully): name = "Name"
            match nvs.nested.pop().unwrap().into_value() {
                syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                    if !nv.path.is_ident("name") {
                        panic!("Expected: `name`; found: {:?}", nv.lit);
                    }
                    nv
                }
                _ => panic!("Invalid nested attribute format"),
            }
        }
        Ok(syn::Meta::Path(_)) => {
            // inside of #[] there was just an identifier (`#[abi]`)
            // include to parsing without attributes
            return None;
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
    };

    match meta.lit {
        syn::Lit::Str(s) => {
            name = Some(s.value());
        }
        lit => panic!("Expected string, found {:?}", lit),
    }

    return name;
}
