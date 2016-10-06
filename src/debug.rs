use ast;
use attr;
use bound;
use quote;
use syn::{self, aster};

pub fn derive(input: &ast::Input, debug: &attr::InputDebug) -> quote::Tokens {
    fn make_variant_data(
        variant_name: quote::Tokens,
        variant_name_as_str: &str,
        style: ast::Style,
        fields: &[ast::Field],
        transparent: bool,
    ) -> quote::Tokens {
        match style {
            ast::Style::Struct => {
                let mut field_pats = quote::Tokens::new();
                let mut field_prints = quote::Tokens::new();

                for (n, f) in fields.iter().enumerate() {
                    let name = f.ident.as_ref().unwrap();
                    field_pats.append(&format!("{}: ref __arg_{},", name, n));

                    if !f.attrs.debug.as_ref().map_or(false, |d| d.ignore) {
                        field_prints.append(&format!("let _ = builder.field(\"{}\", &__arg_{});", name, n));
                    }
                }

                quote!(
                    #variant_name { #field_pats } => {
                        let mut builder = f.debug_struct(#variant_name_as_str);
                        #field_prints
                        builder.finish()
                    }
                )
            }
            ast::Style::Tuple if transparent => {
                quote!(
                    #variant_name( ref __arg_0 ) => {
                        ::std::fmt::Debug::fmt(__arg_0, f)
                    }
                )
            }
            ast::Style::Tuple => {
                let mut field_pats = quote::Tokens::new();
                let mut field_prints = quote::Tokens::new();

                for (n, f) in fields.iter().enumerate() {
                    field_pats.append(&format!("ref __arg_{},", n));

                    if !f.attrs.debug.as_ref().map_or(false, |d| d.ignore) {
                        field_prints.append(&format!("let _ = builder.field(&__arg_{});", n));
                    }
                }

                quote!(
                    #variant_name( #field_pats ) => {
                        let mut builder = f.debug_tuple(#variant_name_as_str);
                        #field_prints
                        builder.finish()
                    }
                )
            }
            ast::Style::Unit => {
                quote!(
                    #variant_name => f.write_str(#variant_name_as_str)
                )
            }
        }
    }

    let name = &input.ident;

    let arms = match input.body {
        ast::Body::Enum(ref data) => {
            let arms = data.iter().map(|variant| {
                let vname = &variant.ident;
                let vname_as_str = vname.as_ref();
                let transparent = variant.attrs.debug.as_ref().map_or(false, |debug| debug.transparent);

                make_variant_data(quote!(#name::#vname), vname_as_str, variant.style, &variant.fields, transparent)
            });

            quote!(#(arms),*)
        }
        ast::Body::Struct(style, ref vd) => {
            let arms = make_variant_data(quote!(#name), name.as_ref(), style, vd, debug.transparent);

            quote!(#arms)
        }
    };

    let debug_trait_path = debug_trait_path();
    let impl_generics = build_impl_generics(input);
    let where_clause = &impl_generics.where_clause;

    let ty = syn::aster::ty().path()
                             .segment(name.clone())
                             .with_generics(impl_generics.clone())
                             .build()
                             .build();

    quote!(
        impl #impl_generics #debug_trait_path for #ty #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self {
                    #arms
                }
            }
        }
    )
}

/// Make generic with all the generics in the input, plus a bound `T: Debug` for each generic field
/// type that will be shown.
fn build_impl_generics(item: &ast::Input) -> syn::Generics {
    let generics = bound::without_defaults(&item.generics);
    let generics = bound::with_where_predicates_from_fields(
        item, &generics,
        |attrs| attrs.debug_bound());

    match item.attrs.debug_bound() {
        Some(predicates) => {
            bound::with_where_predicates(&generics, predicates)
        }
        None => {
            bound::with_bound(item, &generics, needs_deserialize_bound, &debug_trait_path())
        }
    }
}

fn needs_deserialize_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_debug() && attrs.debug_bound().is_none()
}

/// Return the path of the `Debug` trait, that is `::std::fmt::Debug`.
fn debug_trait_path() -> syn::Path {
    aster::path().global().ids(&["std", "fmt", "Debug"]).build()
}
