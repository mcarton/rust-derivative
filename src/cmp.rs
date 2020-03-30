// https://github.com/rust-lang/rust/issues/13101

use proc_macro2;

use ast;
use attr;
use matcher;
use paths;
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
pub fn derive_partial_eq(input: &ast::Input) -> proc_macro2::TokenStream {
    let discriminant_cmp = if let ast::Body::Enum(_) = input.body {
        let discriminant_path = paths::discriminant_path();

        quote!((#discriminant_path(&*self) == #discriminant_path(&*other)))
    } else {
        quote!(true)
    };
    let body = matcher::Matcher::new(matcher::BindingStyle::Ref).build_2_arms(
        (input, "__self"),
        (input, "__other"),
        |_, _, _, (left_variant, right_variant)| {
            let cmp = left_variant.iter().zip(&right_variant).map(|(o, i)| {
                let outer_name = &o.ident;
                let inner_name = &i.ident;

                if o.field.attrs.ignore_partial_eq() {
                    None
                } else if let Some(compare_fn) = o.field.attrs.partial_eq_compare_with() {
                    Some(quote!(&& #compare_fn(#outer_name, #inner_name)))
                } else {
                    Some(quote!(&& #outer_name == #inner_name))
                }
            });

            quote!(true #(#cmp)*)
        },
    );

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

    let match_fields = if input.is_trivial_enum() {
        quote!(true)
    } else {
        quote! {
            match (&*self, &*other) {
                #body
                _ => unreachable!(),
            }
        }
    };

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #partial_eq_trait_path for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                #discriminant_cmp && #match_fields
            }
        }
    }
}

/// Derive `PartialOrd` for `input`.
pub fn derive_partial_ord(input: &ast::Input, errors: &mut proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if let ast::Body::Enum(_) = input.body {
        if !input.attrs.partial_ord_on_enum() {
            let message = "can't use `#[derivative(PartialOrd)]` on an enumeration without \
            `feature_allow_slow_enum`; see the documentation for more details";
            errors.extend(
                syn::Error::new(input.span, message).to_compile_error()
            );
        }
    }

    let option_path = option_path();
    let ordering_path = ordering_path();

    let body = matcher::Matcher::new(matcher::BindingStyle::Ref).build_arms(
        input,
        "__self",
        |_, n, _, _, _, outer_bis| {
            let body = matcher::Matcher::new(matcher::BindingStyle::Ref).build_arms(
                input,
                "__other",
                |_, m, _, _, _, inner_bis| match n.cmp(&m) {
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
                },
            );

            quote! {
                match *other {
                    #body
                }

            }
        },
    );

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

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #partial_ord_trait_path for #name #ty_generics #where_clause {
            fn partial_cmp(&self, other: &Self) -> #option_path<#ordering_path> {
                match *self {
                    #body
                }
            }
        }
    }
}

/// Derive `Ord` for `input`.
pub fn derive_ord(input: &ast::Input, errors: &mut proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if let ast::Body::Enum(_) = input.body {
        if !input.attrs.ord_on_enum() {
            let message = "can't use `#[derivative(Ord)]` on an enumeration without \
            `feature_allow_slow_enum`; see the documentation for more details";
            errors.extend(
                syn::Error::new(input.span, message).to_compile_error()
            );
        }
    }

    let ordering_path = ordering_path();

    let body = matcher::Matcher::new(matcher::BindingStyle::Ref).build_arms(
        input,
        "__self",
        |_, n, _, _, _, outer_bis| {
            let body = matcher::Matcher::new(matcher::BindingStyle::Ref).build_arms(
                input,
                "__other",
                |_, m, _, _, _, inner_bis| match n.cmp(&m) {
                    ::std::cmp::Ordering::Less => quote!(#ordering_path::Less),
                    ::std::cmp::Ordering::Greater => quote!(#ordering_path::Greater),
                    ::std::cmp::Ordering::Equal => {
                        let equal_path = quote!(#ordering_path::Equal);
                        outer_bis
                            .iter()
                            .rev()
                            .zip(inner_bis.into_iter().rev())
                            .fold(quote!(#equal_path), |acc, (o, i)| {
                                let outer_name = &o.ident;
                                let inner_name = &i.ident;

                                if o.field.attrs.ignore_ord() {
                                    acc
                                } else {
                                    let cmp_fn = o
                                        .field
                                        .attrs
                                        .ord_compare_with()
                                        .map(|f| quote!(#f))
                                        .unwrap_or_else(|| {
                                            let path = ord_trait_path();
                                            quote!(#path::cmp)
                                        });

                                    quote!(match #cmp_fn(&(*#outer_name), &(*#inner_name)) {
                                       #equal_path => #acc,
                                        __derive_ordering_other => __derive_ordering_other,
                                    })
                                }
                            })
                    }
                },
            );

            quote! {
                match *other {
                    #body
                }

            }
        },
    );

    let name = &input.ident;

    let ord_trait_path = ord_trait_path();
    let generics = utils::build_impl_generics(
        input,
        &ord_trait_path,
        needs_ord_bound,
        |field| field.ord_bound(),
        |input| input.ord_bound(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #ord_trait_path for #name #ty_generics #where_clause {
            fn cmp(&self, other: &Self) -> #ordering_path {
                match *self {
                    #body
                }
            }
        }
    }
}

fn needs_partial_eq_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_partial_eq() && attrs.partial_eq_bound().is_none()
}

fn needs_partial_ord_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_partial_ord() && attrs.partial_ord_bound().is_none()
}

fn needs_ord_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_ord() && attrs.ord_bound().is_none()
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

/// Return the path of the `Ord` trait, that is `::std::cmp::Ord`.
fn ord_trait_path() -> syn::Path {
    if cfg!(feature = "use_core") {
        parse_quote!(::core::cmp::Ord)
    } else {
        parse_quote!(::std::cmp::Ord)
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