#![feature(rustc_macro, rustc_macro_lib)]

extern crate rustc_macro;
extern crate syn;

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syntex_syntax as syntax;

use rustc_macro::TokenStream;
use std::str::FromStr;

fn derivative_attribute(attr: &syn::Attribute) -> Option<&[syn::MetaItem]> {
    match attr.value {
        syn::MetaItem::List(ref name, ref mis) if name == "derivative" => {
            Some(mis)
        }
        syn::MetaItem::Word(..) |
        syn::MetaItem::NameValue(..) |
        syn::MetaItem::List(..) => None,
    }
}

fn collect_derive_attrs(mut input: syn::MacroInput) -> (syn::MacroInput, Vec<syn::MetaItem>) {
    let mut attrs = Vec::new();

    input.attrs.retain(|attr| {
        if let Some(mis) = derivative_attribute(&attr) {
            attrs.extend_from_slice(mis);
            false
        } else {
            true
        }
    });

    (input, attrs)
}

fn ignored_traits(attrs: &[syn::Attribute]) -> Vec<&str> {
    attrs.iter().filter_map(derivative_attribute).flat_map(|a| a).map(|attr| {
        match *attr {
            syn::MetaItem::List(ref name, ref mis) if name == "ignore_for" => {
                mis.iter().map(|mi| {
                    if let syn::MetaItem::Word(ref name) = *mi {
                        name.as_ref()
                    } else {
                        panic!()
                    }
                }).collect()
            }
            _ => Vec::new(),
        }
    }).flat_map(|s| s).collect()
}

fn derive_debug(input: &syn::MacroInput) -> String {
    fn make_variant_data(
        variant_name: quote::Tokens,
        variant_name_as_str: &str,
        data: &syn::VariantData
    ) -> quote::Tokens {
        match *data {
            syn::VariantData::Struct(ref fields) => {
                let mut field_pats = quote::Tokens::new();
                let mut field_prints = quote::Tokens::new();

                for (n, f) in fields.iter().enumerate() {
                    let name = f.ident.as_ref().unwrap();
                    field_pats.append(&format!("{}: ref __arg_{},", name, n));

                    if !ignored_traits(&f.attrs).contains(&"Debug") {
                        field_prints.append(&format!(".field(\"{}\", &__arg_{})", name, n));
                    }
                }

                quote!(
                    #variant_name { #field_pats } => f.debug_struct(#variant_name_as_str) #field_prints .finish()
                )
            }
            syn::VariantData::Tuple(ref fields) => {
                let mut field_pats = quote::Tokens::new();
                let mut field_prints = quote::Tokens::new();

                for (n, f) in fields.iter().enumerate() {
                    field_pats.append(&format!("ref __arg_{},", n));

                    if !ignored_traits(&f.attrs).contains(&"Debug") {
                        field_prints.append(&format!(".field(&__arg_{})", n));
                    }
                }

                quote!(
                    #variant_name( #field_pats ) =>
                        f.debug_tuple(#variant_name_as_str) #field_prints .finish()
                )
            }
            syn::VariantData::Unit => {
                quote!(
                    #variant_name => f.write_str(#variant_name_as_str)
                )
            }
        }
    }

    let name = &input.ident;

    let arms = match input.body {
        syn::Body::Enum(ref data) => {
            let arms = data.iter().map(|field| {
                let fname = &field.ident;
                let fname_as_str = fname.as_ref();

                make_variant_data(quote!(#name::#fname), fname_as_str, &field.data)
            });

            quote!(#(arms),*)
        }
        syn::Body::Struct(ref vd) => {
            let arms = make_variant_data(quote!(#name), name.as_ref(), vd);

            quote!(#arms)
        }
    };

    quote!(
        impl ::std::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self {
                    #arms
                }
            }
        }
    ).to_string()
}

fn derive_impl_from_name(input: &syn::MacroInput, attr: &str) -> String {
    match attr {
        "Debug" => derive_debug(input),
        _ => panic!("Unknown trait `{}`", attr),
    }
}

fn derive_impl(input: &syn::MacroInput, attr: syn::MetaItem) -> String {
    if let syn::MetaItem::Word(name) = attr {
        derive_impl_from_name(input, name.as_ref())
    } else {
        panic!("`#[derivative]` expected just a name");
    }
}

#[rustc_macro_derive(Derivative)]
pub fn derivative(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input(&input.to_string()).unwrap();

    let (input, attrs) = collect_derive_attrs(input);
    println!("{:?}", input);

    let mut output = quote!(#input).to_string();

    output.extend(attrs.into_iter().map(|attr| {
        derive_impl(&input, attr)
    }));

    println!("{:?}", output);

    TokenStream::from_str(&output).unwrap()
}
