// https://github.com/rust-lang/rust/issues/13101

use ast;
use matcher;
use quote;
use syn;
use utils;

/// Derive `Eq` for `input`.
pub fn derive_eq(input: &ast::Input) -> quote::Tokens {
    let name = input.ident;
    let eq_trait_path = eq_trait_path();

    let (impl_generics, path_args) = utils::build_generics(
        input,
        &eq_trait_path,
        |attrs| attrs.eq_bound().is_none(),
        |field| field.eq_bound(),
        |input| input.eq_bound(),
    );
    let where_clause = &impl_generics.where_clause;

    let type_ = syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::PathSegment {
            ident: name,
            arguments: path_args,
        }.into(),
    });

    quote! {
        impl #impl_generics #eq_trait_path for #type_ #where_clause {}
    }
}

/// Derive `PartialEq` for `input`.
pub fn derive_partial_eq(input: &ast::Input) -> Result<quote::Tokens, String> {
    if let ast::Data::Enum(_) = input.data {
        if !input.attrs.partial_eq_on_enum() {
            return Err(
                "can't use `#[derivative(PartialEq)]` on an enumeration without \
                `feature_allow_slow_enum`; see the documentation for more details".to_string()
            );
        }
    }

    let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
        .with_name("__self".into())
        .build_arms(input, |_, outer_arm_name, _, _, outer_bis| {
            let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
                .with_name("__other".into())
                .build_arms(input, |_, inner_arm_name, _, _, inner_bis| {
                    if outer_arm_name == inner_arm_name {
                        let cmp = outer_bis.iter().zip(inner_bis).filter_map(|(o, i)| {
                            let outer_name = o.ident;
                            let inner_name = i.ident;

                            if o.field.attrs.ignore_partial_eq() {
                                None
                            } else if let Some(compare_fn) = o.field.attrs.partial_eq_compare_with() {
                                Some(quote!(#compare_fn(#outer_name, #inner_name)))
                            } else {
                                Some(quote!(#outer_name == #inner_name))
                            }
                        });

                        quote!(true #(&& #cmp)*)
                    } else {
                        quote!(false)
                    }
                });

            quote! {
                match *other {
                    #body
                }
            }
        });

    let name = input.ident;
    let partial_eq_trait_path = partial_eq_trait_path();

    let (impl_generics, path_args) = utils::build_generics(
        input,
        &partial_eq_trait_path,
        |attrs| attrs.partial_eq_bound().is_none(),
        |field| field.partial_eq_bound(),
        |input| input.partial_eq_bound(),
    );
    let where_clause = &impl_generics.where_clause;

    let type_ = syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::PathSegment {
            ident: name,
            arguments: path_args,
        }.into(),
    });

    Ok(quote! {
        impl #impl_generics #partial_eq_trait_path for #type_ #where_clause {
            fn eq(&self, other: &Self) -> bool {
                match *self {
                    #body
                }
            }
        }
    })
}

/// Return the path of the `Eq` trait, that is `::std::cmp::Eq`.
fn eq_trait_path() -> syn::Path {
    parse_quote! { ::std::cmp::Eq }
}

/// Return the path of the `PartialEq` trait, that is `::std::cmp::PartialEq`.
fn partial_eq_trait_path() -> syn::Path {
    parse_quote! { ::std::cmp::PartialEq }
}
