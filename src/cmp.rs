// https://github.com/rust-lang/rust/issues/13101

use ast;
use matcher;
use quote;
use syn::{self, aster};
use utils;

/// Derive `Eq` for `input`.
pub fn derive_eq(input: &ast::Input) -> quote::Tokens {
    let name = &input.ident;

    let eq_trait_path = eq_trait_path();
    let impl_generics = utils::build_impl_generics(
        input,
        &eq_trait_path,
        |attrs| attrs.eq_bound().is_none(),
        |field| field.eq_bound(),
        |input| input.eq_bound(),
    );
    let where_clause = &impl_generics.where_clause;

    let ty = syn::aster::ty().path()
                             .segment(name.clone())
                             .with_generics(impl_generics.clone())
                             .build()
                             .build();

    quote! {
        impl #impl_generics #eq_trait_path for #ty #where_clause {}
    }
}

/// Derive `PartialEq` for `input`.
pub fn derive_partial_eq(input: &ast::Input) -> quote::Tokens {
    let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
        .with_name("__self".into())
        .build_arms(input, |_arm_name, _style, _attrs, outer_bis| {
            let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
                .with_name("__other".into())
                .build_arms(input, |_arm_name, _style, _attrs, inner_bis| {
                    let cmp = outer_bis.iter().zip(inner_bis).map(|(o, i)| {
                        let outer_name = &o.ident;
                        let inner_name = &i.ident;

                        if o.field.attrs.ignore_partial_eq() {
                            None
                        } else {
                            Some(quote!(&& #outer_name == #inner_name))
                        }
                    });

                    quote!(true #(#cmp)*)
                }
            );

            quote! {
                match *other {
                    #body
                }
            }
        }
    );

    let name = &input.ident;

    let partial_eq_trait_path = partial_eq_trait_path();
    let impl_generics = utils::build_impl_generics(
        input,
        &partial_eq_trait_path,
        |attrs| attrs.partial_eq_bound().is_none(),
        |field| field.partial_eq_bound(),
        |input| input.partial_eq_bound(),
    );
    let where_clause = &impl_generics.where_clause;

    let ty = syn::aster::ty().path()
                             .segment(name.clone())
                             .with_generics(impl_generics.clone())
                             .build()
                             .build();

    quote! {
        impl #impl_generics #partial_eq_trait_path for #ty #where_clause {
            fn eq(&self, other: &Self) -> bool {
                match *self {
                    #body
                }
            }
        }
    }
}

/// Return the path of the `Eq` trait, that is `::std::cmp::Eq`.
fn eq_trait_path() -> syn::Path {
    aster::path().global().ids(&["std", "cmp", "Eq"]).build()
}

/// Return the path of the `PartialEq` trait, that is `::std::cmp::PartialEq`.
fn partial_eq_trait_path() -> syn::Path {
    aster::path().global().ids(&["std", "cmp", "PartialEq"]).build()
}
