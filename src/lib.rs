extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

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

use proc_macro::TokenStream;

fn derive_impls(input: ast::Input) -> Result<quote::Tokens, String> {
    let mut tokens = quote::Tokens::new();

    if input.attrs.clone.is_some() {
        tokens.append_all(clone::derive_clone(&input));
    }
    if input.attrs.copy.is_some() {
        tokens.append_all(clone::derive_copy(&input)?);
    }
    if input.attrs.debug.is_some() {
        tokens.append_all(debug::derive(&input));
    }
    if let Some(ref default) = input.attrs.default {
        tokens.append_all(default::derive(&input, default));
    }
    if input.attrs.eq.is_some() {
        tokens.append_all(cmp::derive_eq(&input));
    }
    if input.attrs.hash.is_some() {
        tokens.append_all(hash::derive(&input));
    }
    if input.attrs.partial_eq.is_some() {
        tokens.append_all(cmp::derive_partial_eq(&input)?);
    }

    Ok(tokens)
}

#[cfg_attr(not(test), proc_macro_derive(Derivative, attributes(derivative)))]
pub fn derivative(input: TokenStream) -> TokenStream {
    fn detail(input: TokenStream) -> Result<TokenStream, String> {
        let input = syn::parse(input).map_err(|e| e.to_string())?;
        let parsed = ast::Input::from_ast(&input)?;
        let output = derive_impls(parsed)?;
        Ok(output.into())
    }

    match detail(input) {
        Ok(output) => output,
        Err(e) => panic!(e),
    }
}
