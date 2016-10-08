#![feature(rustc_macro, rustc_macro_lib)]

extern crate rustc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

mod ast;
mod attr;
mod bound;
mod debug;
mod default;
mod utils;

use rustc_macro::TokenStream;

fn derive_impls(input: &ast::Input) -> Result<quote::Tokens, String> {
    let mut tokens = quote::Tokens::new();

    if let Some(ref debug) = input.attrs.debug {
        tokens.append(&debug::derive(input, debug).to_string());
    }
    if let Some(ref default) = input.attrs.default {
        tokens.append(&default::derive(input, default).to_string());
    }

    Ok(tokens)
}

#[cfg_attr(not(test), rustc_macro_derive(Derivative))]
pub fn derivative(input: TokenStream) -> TokenStream {
    fn detail(input: TokenStream) -> Result<TokenStream, String> {
        let mut input = try!(syn::parse_macro_input(&input.to_string()));
        let mut output = {
            let parsed = try!(ast::Input::from_ast(&input));

            try!(derive_impls(&parsed))
        };

        utils::remove_derivative_attrs(&mut input);
        output.append(&quote!(#input).to_string());

        Ok(output.to_string().parse().unwrap())
    }

    detail(input).unwrap()
}
