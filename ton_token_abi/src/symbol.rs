use std::fmt::Display;
use syn::{Ident, Path};

macro_rules! define_symbols(
    ($($name:ident => $value:literal),*,) => {
        $(pub const $name: Symbol = Symbol($value));*;
    };
);

define_symbols! {
    // main macro name
    ABI => "abi",

    // container attributes
    PLAIN => "plain",

    // field attributes
    NAME => "name",
    PARSE_WITH => "parse_with",
    PARSE_TYPE => "parse_type",
}

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

impl Symbol {
    pub fn inner(&self) -> &'static str {
        self.0
    }
}

impl PartialEq<Symbol> for Ident {
    fn eq(&self, other: &Symbol) -> bool {
        self == other.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, other: &Symbol) -> bool {
        *self == other.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, other: &Symbol) -> bool {
        self.is_ident(other.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, other: &Symbol) -> bool {
        self.is_ident(other.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
