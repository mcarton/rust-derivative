// https://github.com/rust-lang/rust/issues/13101

use proc_macro2;

use ast;
use attr;
use matcher;
use syn;
use utils;

/// Derive `Eq` for `input`.
pub fn derive_eq(input: &ast::Input) -> proc_macro2::TokenStream {
    let name = &input.ident;

    let eq_trait_path = eq_trait_path();
    let generics = utils::build_impl_generics(
        input,
        &eq_trait_path,
        needs_eq_bound,
        |field| field.eq_bound(),
        |input| input.eq_bound(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #eq_trait_path for #name #ty_generics #where_clause {}
    }
}

/// Derive `PartialEq` for `input`.
pub fn derive_partial_eq(input: &ast::Input) -> Result<proc_macro2::TokenStream, String> {
    if let ast::Body::Enum(_) = input.body {
        if !input.attrs.partial_eq_on_enum() {
            return Err(
                "can't use `#[derivative(PartialEq)]` on an enumeration without \
                 `feature_allow_slow_enum`; see the documentation for more details"
                    .into(),
            );
        }
    }

    let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
        .with_name("__self".into())
        .build_arms(input, |_, _, outer_arm_name, _, _, outer_bis| {
            let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
                .with_name("__other".into())
                .build_arms(input, |_, _, inner_arm_name, _, _, inner_bis| {
                    if outer_arm_name == inner_arm_name {
                        let cmp = outer_bis.iter().zip(inner_bis).map(|(o, i)| {
                            let outer_name = &o.ident;
                            let inner_name = &i.ident;

                            if o.field.attrs.ignore_partial_eq() {
                                None
                            } else if let Some(compare_fn) = o.field.attrs.partial_eq_compare_with()
                            {
                                Some(quote!(&& #compare_fn(#outer_name, #inner_name)))
                            } else {
                                Some(quote!(&& #outer_name == #inner_name))
                            }
                        });

                        quote!(true #(#cmp)*)
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

    let name = &input.ident;

    let partial_eq_trait_path = partial_eq_trait_path();
    let generics = utils::build_impl_generics(
        input,
        &partial_eq_trait_path,
        needs_partial_eq_bound,
        |field| field.partial_eq_bound(),
        |input| input.partial_eq_bound(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #partial_eq_trait_path for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                match *self {
                    #body
                }
            }
        }
    })
}

/// Derive `PartialOrd` for `input`.
pub fn derive_partial_ord(input: &ast::Input) -> Result<proc_macro2::TokenStream, String> {
    if let ast::Body::Enum(_) = input.body {
        if !input.attrs.partial_ord_on_enum() {
            return Err(
                "can't use `#[derivative(PartialOrd)]` on an enumeration without \
                 `feature_allow_slow_enum`; see the documentation for more details"
                    .into(),
            );
        }
    }

    let option_path = option_path();
    let ordering_path = ordering_path();

    let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
        .with_name("__self".into())
        .build_arms(input, |_, n, _, _, _, outer_bis| {
            let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
                .with_name("__other".into())
                .build_arms(input, |_, m, _, _, _, inner_bis| match n.cmp(&m) {
                    ::std::cmp::Ordering::Less => quote!(#option_path::Some(#ordering_path::Less)),
                    ::std::cmp::Ordering::Greater => {
                        quote!(#option_path::Some(#ordering_path::Greater))
                    }
                    ::std::cmp::Ordering::Equal => {
                        let equal_path = quote!(#ordering_path::Equal);
                        outer_bis
                            .iter()
                            .rev()
                            .zip(inner_bis.into_iter().rev())
                            .fold(quote!(#option_path::Some(#equal_path)), |acc, (o, i)| {
                                let outer_name = &o.ident;
                                let inner_name = &i.ident;

                                if o.field.attrs.ignore_partial_ord() {
                                    acc
                                } else {
                                    let cmp_fn = o
                                        .field
                                        .attrs
                                        .partial_ord_compare_with()
                                        .map(|f| quote!(#f))
                                        .unwrap_or_else(|| {
                                            let path = partial_ord_trait_path();
                                            quote!(#path::partial_cmp)
                                        });

                                    quote!(match #cmp_fn(&(*#outer_name), &(*#inner_name)) {
                                        #option_path::Some(#equal_path) => #acc,
                                        __derive_ordering_other => __derive_ordering_other,
                                    })
                                }
                            })
                    }
                });

            quote! {
                match *other {
                    #body
                }

            }
        });

    let name = &input.ident;

    let partial_ord_trait_path = partial_ord_trait_path();
    let generics = utils::build_impl_generics(
        input,
        &partial_ord_trait_path,
        needs_partial_ord_bound,
        |field| field.partial_ord_bound(),
        |input| input.partial_ord_bound(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #partial_ord_trait_path for #name #ty_generics #where_clause {
            fn partial_cmp(&self, other: &Self) -> #option_path<#ordering_path> {
                match *self {
                    #body
                }
            }
        }
    })
}

fn needs_partial_eq_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_partial_eq() && attrs.partial_eq_bound().is_none()
}

fn needs_partial_ord_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_partial_ord() && attrs.partial_ord_bound().is_none()
}

fn needs_eq_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_partial_eq() && attrs.eq_bound().is_none()
}

/// Return the path of the `Eq` trait, that is `::std::cmp::Eq`.
fn eq_trait_path() -> syn::Path {
    if cfg!(feature = "use_core") {
        parse_quote!(::core::cmp::Eq)
    } else {
        parse_quote!(::std::cmp::Eq)
    }
}

/// Return the path of the `PartialEq` trait, that is `::std::cmp::PartialEq`.
fn partial_eq_trait_path() -> syn::Path {
    if cfg!(feature = "use_core") {
        parse_quote!(::core::cmp::PartialEq)
    } else {
        parse_quote!(::std::cmp::PartialEq)
    }
}

/// Return the path of the `PartialOrd` trait, that is `::std::cmp::PartialOrd`.
fn partial_ord_trait_path() -> syn::Path {
    if cfg!(feature = "use_core") {
        parse_quote!(::core::cmp::PartialOrd)
    } else {
        parse_quote!(::std::cmp::PartialOrd)
    }
}

/// Return the path of the `Option` trait, that is `::std::option::Option`.
fn option_path() -> syn::Path {
    if cfg!(feature = "use_core") {
        parse_quote!(::core::option::Option)
    } else {
        parse_quote!(::std::option::Option)
    }
}

/// Return the path of the `Ordering` trait, that is `::std::cmp::Ordering`.
fn ordering_path() -> syn::Path {
    if cfg!(feature = "use_core") {
        parse_quote!(::core::cmp::Ordering)
    } else {
        parse_quote!(::std::cmp::Ordering)
    }
}
