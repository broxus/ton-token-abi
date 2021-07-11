use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Meta::*;
use syn::NestedMeta::*;

use crate::parsing_context::*;
use crate::symbol::*;

pub struct Container {
    pub plain: bool,
}

impl Container {
    pub fn from_ast(cx: &ParsingContext, input: &syn::DeriveInput) -> Self {
        let mut plain = BoolAttr::none(cx, PLAIN);

        for (from, meta_item) in input
            .attrs
            .iter()
            .flat_map(|attr| get_meta_items(&cx, attr))
            .flat_map(|item| item.into_iter())
        {
            match (from, &meta_item) {
                (AttrFrom::Abi, Meta(Path(word))) if word == PLAIN => plain.set_true(word),
                (AttrFrom::Abi, _) => {}
            }
        }

        Self { plain: plain.get() }
    }
}

pub struct Field {
    pub name: Option<String>,
    pub parse_type: Option<ParseType>,
}

impl Field {
    pub fn from_ast(cx: &ParsingContext, _index: usize, input: &syn::Field) -> Self {
        let mut name = Attr::none(cx, NAME);
        let mut parse_type = Attr::none(cx, PARSE_TYPE);

        for (from, meta_item) in input
            .attrs
            .iter()
            .flat_map(|attr| get_meta_items(&cx, attr))
            .flat_map(|item| item.into_iter())
        {
            match (from, &meta_item) {
                (AttrFrom::Abi, Meta(NameValue(m))) if m.path == NAME => {
                    if let Ok(s) = get_lit_str(cx, NAME, &m.lit) {
                        name.set(&m.path, s.value());
                    }
                }
                (AttrFrom::Abi, Lit(lit)) => {
                    if let Ok(s) = get_lit_str_simple(lit) {
                        let pt = ParseType::from(s.value().as_str());
                        if pt != ParseType::NONE {
                            parse_type.set(lit, pt);
                        } else {
                            cx.error_spanned_by(lit, "unknown parse type")
                        }
                    }
                }
                (AttrFrom::Abi, _) => {}
            }
        }

        Self {
            name: name.get(),
            parse_type: parse_type.get(),
        }
    }
}

fn get_lit_str_simple(lit: &syn::Lit) -> Result<&syn::LitStr, ()> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        Err(())
    }
}

fn get_lit_str<'a>(
    cx: &ParsingContext,
    attr_name: Symbol,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    get_lit_str_special(cx, attr_name, attr_name, lit)
}

fn get_lit_str_special<'a>(
    cx: &ParsingContext,
    attr_name: Symbol,
    path_name: Symbol,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        cx.error_spanned_by(
            lit,
            format!(
                "expected {} attribute to be a string: `{} = \"...\"`",
                attr_name, path_name
            ),
        );
        Err(())
    }
}

fn get_meta_items(
    cx: &ParsingContext,
    attr: &syn::Attribute,
) -> Result<Vec<(AttrFrom, syn::NestedMeta)>, ()> {
    let attr_from = if attr.path == ABI {
        AttrFrom::Abi
    } else {
        return Ok(Vec::new());
    };

    match attr.parse_meta() {
        Ok(List(meta)) => Ok(meta
            .nested
            .into_iter()
            .map(|meta| (attr_from, meta))
            .collect()),
        Ok(other) => {
            cx.error_spanned_by(other, format!("expected #[{}(...)]", attr_from));
            Err(())
        }
        Err(err) => {
            cx.syn_error(err);
            Err(())
        }
    }
}

#[derive(Copy, Clone)]
enum AttrFrom {
    Abi,
}

impl std::fmt::Display for AttrFrom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttrFrom::Abi => f.write_str(ABI.inner()),
        }
    }
}

struct Attr<'c, T> {
    cx: &'c ParsingContext,
    name: Symbol,
    tokens: TokenStream,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(cx: &'c ParsingContext, name: Symbol) -> Self {
        Attr {
            cx,
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    fn set<A: ToTokens>(&mut self, object: A, value: T) {
        let tokens = object.into_token_stream();

        if self.value.is_some() {
            self.cx
                .error_spanned_by(tokens, format!("duplicate abi attribute `{}`", self.name));
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    #[allow(dead_code)]
    fn set_opt<A: ToTokens>(&mut self, object: A, value: Option<T>) {
        if let Some(value) = value {
            self.set(object, value);
        }
    }

    #[allow(dead_code)]
    fn set_if_none(&mut self, value: T) {
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    fn get(self) -> Option<T> {
        self.value
    }

    #[allow(dead_code)]
    fn get_with_tokens(self) -> Option<(TokenStream, T)> {
        match self.value {
            Some(value) => Some((self.tokens, value)),
            None => None,
        }
    }
}

struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    fn none(cx: &'c ParsingContext, name: Symbol) -> Self {
        BoolAttr(Attr::none(cx, name))
    }

    fn set_true<A: ToTokens>(&mut self, object: A) {
        self.0.set(object, ());
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

#[derive(PartialEq)]
pub enum ParseType {
    UINT8,
    UINT16,
    UINT32,
    UINT64,
    UINT128,
    UINT256,
    BOOL,
    NONE,
}

impl ParseType {
    fn from(input: &str) -> ParseType {
        match input {
            "uint8" => ParseType::UINT8,
            "uint16" => ParseType::UINT16,
            "uint32" => ParseType::UINT32,
            "uint64" => ParseType::UINT64,
            "uint128" => ParseType::UINT128,
            "uint256" => ParseType::UINT256,
            "bool" => ParseType::BOOL,
            _ => ParseType::NONE,
        }
    }
}
