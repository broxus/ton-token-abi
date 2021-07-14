mod abi;
mod ast;
mod attr;
mod parsing_context;
mod symbol;

use self::abi::*;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(TokenAbi, attributes(abi))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    impl_derive(input).unwrap_or_else(to_compile_errors).into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
