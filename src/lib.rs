#![feature(rustc_macro, rustc_macro_lib)]

extern crate rustc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

mod ast;
mod attr;
mod bound;
mod debug;
mod utils;

use rustc_macro::TokenStream;

fn derive_impls(input: &ast::Input) -> quote::Tokens {
    let mut tokens = quote::Tokens::new();

    if let Some(ref debug) = input.attrs.debug {
        tokens.append(&debug::derive(input, debug).to_string());
    }

    tokens
}

#[rustc_macro_derive(Derivative)]
pub fn derivative(input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input(&input.to_string()).unwrap();
    let mut output = {
        let parsed = ast::Input::from_ast(&(), &input);

        derive_impls(&parsed)
    };

    utils::remove_derivative_attrs(&mut input);
    output.append(&quote!(#input).to_string());

    output.to_string().parse().unwrap()
}
