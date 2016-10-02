#![feature(rustc_macro, rustc_macro_lib)]

extern crate rustc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

mod debug;
mod utils;

use rustc_macro::TokenStream;

fn derive_impl_from_name(input: &syn::MacroInput, attr: &str, value: Option<syn::Lit>) -> quote::Tokens {
    match attr {
        "Debug" => debug::derive(input, value),
        _ => panic!("Unknown trait `{}`", attr),
    }
}

fn derive_impl(input: &syn::MacroInput, attr: syn::MetaItem) -> quote::Tokens {
    match attr {
        syn::MetaItem::Word(name) => derive_impl_from_name(input, name.as_ref(), None),
        syn::MetaItem::NameValue(name, lit) => derive_impl_from_name(input, name.as_ref(), Some(lit)),
        _ => panic!("unknown `#[derivative]` attribute"),
    }
}

fn remove_derivative_attrs(input: &mut syn::MacroInput) {
    fn remove_from_vec(attrs: &mut Vec<syn::Attribute>) {
        attrs.retain(|attr| utils::derivative_attribute(&attr).is_none());
    }

    fn remove_from_variant_data(vd: &mut syn::VariantData) {
        match *vd {
            syn::VariantData::Struct(ref mut fields) | syn::VariantData::Tuple(ref mut fields) => {
                for field in fields {
                    remove_from_vec(&mut field.attrs);
                }
            }
            syn::VariantData::Unit => (),
        }
    }

    remove_from_vec(&mut input.attrs);

    match input.body {
        syn::Body::Enum(ref mut variants) => {
            for variant in variants {
                remove_from_vec(&mut variant.attrs);
                remove_from_variant_data(&mut variant.data);
            }
        }
        syn::Body::Struct(ref mut vd) => {
            remove_from_variant_data(vd);
        }
    }
}

#[rustc_macro_derive(Derivative)]
pub fn derivative(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input(&input.to_string()).unwrap();

    println!("{:?}", input);
    let (mut input, attrs) = utils::collect_derive_attrs(input);
    println!("{:?}", input);

    let mut output = quote::Tokens::new();

    output.append_separated(attrs.into_iter().map(|attr| {
        derive_impl(&input, attr)
    }), " ");

    remove_derivative_attrs(&mut input);
    output.append(&quote!(#input).to_string());

    println!("{:?}", output);

    output.to_string().parse().unwrap()
}
