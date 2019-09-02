extern crate proc_macro;

mod ast;
mod attr;
mod bound;
mod clone;
mod cmp;
mod debug;
mod default;
mod hash;
mod matcher;
mod utils;

use proc_macro2::TokenStream;

fn derive_impls(input: &ast::Input) -> Result<TokenStream, String> {
    let mut tokens = TokenStream::new();

    if input.attrs.clone.is_some() {
        tokens.extend(clone::derive_clone(input));
    }
    if input.attrs.copy.is_some() {
        tokens.extend(clone::derive_copy(input));
    }
    if input.attrs.debug.is_some() {
        tokens.extend(debug::derive(input));
    }
    if let Some(default) = &input.attrs.default {
        tokens.extend(default::derive(input, default));
    }
    if input.attrs.eq.is_some() {
        tokens.extend(cmp::derive_eq(input));
    }
    if input.attrs.hash.is_some() {
        tokens.extend(hash::derive(input));
    }
    if input.attrs.partial_eq.is_some() {
        tokens.extend(cmp::derive_partial_eq(input)?);
    }
    if input.attrs.partial_ord.is_some() {
        tokens.extend(cmp::derive_partial_ord(input)?);
    }
    if input.attrs.ord.is_some() {
        tokens.extend(cmp::derive_ord(input)?);
    }

    Ok(tokens)
}

fn detail(input: proc_macro::TokenStream) -> Result<proc_macro::TokenStream, String> {
    let parsed = syn::parse::<syn::DeriveInput>(input).map_err(|e| e.to_string())?;
    let output = derive_impls(&ast::Input::from_ast(&parsed)?)?;
    Ok(output.into())
}

#[cfg_attr(not(test), proc_macro_derive(Derivative, attributes(derivative)))]
pub fn derivative(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match detail(input) {
        Ok(output) => output,
        Err(e) => panic!(e),
    }
}
