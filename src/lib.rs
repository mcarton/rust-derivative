// We need to support Rust 1.34 to stable
#![allow(deprecated)]

#![allow(clippy::mem_replace_with_default)] // needs rustc 1.40
#![allow(clippy::option_as_ref_deref)] // needs rustc 1.40

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

mod ast;
mod attr;
mod bound;
mod clone;
mod cmp;
mod debug;
mod default;
mod hash;
mod matcher;
mod paths;
mod utils;

use proc_macro::TokenStream;

fn derive_impls(
    input: &mut ast::Input,
    errors: &mut proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut tokens = proc_macro2::TokenStream::new();

    if input.attrs.clone.is_some() {
        tokens.extend(clone::derive_clone(input));
    }
    if input.attrs.copy.is_some() {
        tokens.extend(clone::derive_copy(input));
    }
    if input.attrs.debug.is_some() {
        tokens.extend(debug::derive(input));
    }
    if let Some(ref default) = input.attrs.default {
        tokens.extend(default::derive(input, default));
    }
    if input.attrs.eq.is_some() {
        tokens.extend(cmp::derive_eq(input));
    }
    if input.attrs.hash.is_some() {
        tokens.extend(hash::derive(input));
    }
    if input.attrs.partial_eq.is_some() {
        tokens.extend(cmp::derive_partial_eq(input));
    }
    if input.attrs.partial_ord.is_some() {
        tokens.extend(cmp::derive_partial_ord(input, errors));
    }
    if input.attrs.ord.is_some() {
        tokens.extend(cmp::derive_ord(input, errors));
    }

    tokens.extend(std::mem::replace(
        errors,
        Default::default(),
    ));

    tokens
}

#[cfg_attr(not(test), proc_macro_derive(Derivative, attributes(derivative)))]
pub fn derivative(input: TokenStream) -> TokenStream {
    let mut errors = proc_macro2::TokenStream::new();

    let mut output = match syn::parse::<syn::DeriveInput>(input) {
        Ok(parsed) => {
            ast::Input::from_ast(&parsed, &mut errors)
                .map(|mut input| derive_impls(&mut input, &mut errors))
                .unwrap_or_default()
        },
        Err(error) => {
            errors.extend(error.to_compile_error());
            Default::default()
        }
    };

    output.extend(errors);
    output.into()
}