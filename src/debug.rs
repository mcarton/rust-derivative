use ast;
use attr;
use quote;
use syn::{self, aster};
use utils;

pub fn derive(input: &ast::Input, debug: &attr::InputDebug) -> quote::Tokens {
    fn make_variant_data(
        variant_name: quote::Tokens,
        variant_name_as_str: &str,
        style: ast::Style,
        fields: &[ast::Field],
        transparent: bool,
        generics: &syn::Generics,
    ) -> quote::Tokens {
        match style {
            ast::Style::Struct => {
                let mut field_pats = Vec::new();
                let mut field_prints = Vec::new();

                for (n, f) in fields.iter().enumerate() {
                    let name = f.ident.as_ref().expect("A structure field must have a name");
                    let arg_n = syn::Ident::new(format!("__arg_{}", n));

                    field_pats.push(quote!(#name: ref #arg_n));

                    let name = name.as_ref();

                    if let Some(format_fn) = f.attrs.debug_format_with() {
                        let dummy_debug = format_with(f, &arg_n, format_fn, generics.clone());
                        field_prints.push(quote!(
                            #dummy_debug
                            let _ = builder.field(#name, &#arg_n);
                        ));
                    }
                    else if !f.attrs.ignore_debug() {
                        field_prints.push(quote!(let _ = builder.field(#name, &#arg_n);));
                    }
                }

                quote!(
                    #variant_name { #(field_pats),* } => {
                        let mut builder = f.debug_struct(#variant_name_as_str);
                        #(field_prints)*
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
                let mut field_pats = Vec::new();
                let mut field_prints = Vec::new();

                for (n, f) in fields.iter().enumerate() {
                    let arg_n = syn::Ident::new(format!("__arg_{}", n));

                    field_pats.push(quote!(ref #arg_n));

                    if let Some(format_fn) = f.attrs.debug_format_with() {
                        let dummy_debug = format_with(f, &arg_n, format_fn, generics.clone());
                        field_prints.push(quote!(
                            #dummy_debug
                            let _ = builder.field(&#arg_n);
                        ));
                    }
                    else if !f.attrs.ignore_debug() {
                        field_prints.push(quote!(let _ = builder.field(&#arg_n);));
                    }
                }

                quote!(
                    #variant_name( #(field_pats),* ) => {
                        let mut builder = f.debug_tuple(#variant_name_as_str);
                        #(field_prints)*
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

                make_variant_data(quote!(#name::#vname), vname_as_str, variant.style, &variant.fields, transparent, input.generics)
            });

            quote!(#(arms),*)
        }
        ast::Body::Struct(style, ref vd) => {
            let arms = make_variant_data(quote!(#name), name.as_ref(), style, vd, debug.transparent, input.generics);

            quote!(#arms)
        }
    };

    let debug_trait_path = debug_trait_path();
    let impl_generics = utils::build_impl_generics(
        input,
        &debug_trait_path,
        needs_debug_bound,
        |field| field.debug_bound(),
        |input| input.debug_bound(),
    );
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

fn needs_debug_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_debug() && attrs.debug_bound().is_none()
}

/// Return the path of the `Debug` trait, that is `::std::fmt::Debug`.
fn debug_trait_path() -> syn::Path {
    aster::path().global().ids(&["std", "fmt", "Debug"]).build()
}

fn format_with(
    f: &ast::Field,
    arg_n: &syn::Ident,
    format_fn: &syn::Path,
    mut generics: syn::Generics,
) -> quote::Tokens {
    let debug_trait_path = debug_trait_path();

    generics.where_clause.predicates.extend(f.attrs.debug_bound().unwrap_or(&[]).iter().cloned());

    let (_, ctor_generics, _) = generics.split_for_impl();

    generics.lifetimes.push(syn::LifetimeDef::new("'_derivative"));
    for ty in &generics.ty_params {
        let path = aster::path::PathBuilder::new().id(&ty.ident).build();
        generics.where_clause.predicates.push(syn::WherePredicate::BoundPredicate(
            syn::WhereBoundPredicate {
                bound_lifetimes: vec![],
                bounded_ty: syn::Ty::Path(None, path),
                bounds: vec![syn::TyParamBound::Region(syn::Lifetime::new("'_derivative"))],
            }
        ));
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty = f.ty;

    let phantom = &ty_generics.ty_params;

    quote!(
        let #arg_n = {
            struct Dummy #ty_generics (&'_derivative #ty, ::std::marker::PhantomData <(#(phantom),*)>) #where_clause;

            impl #impl_generics #debug_trait_path for Dummy #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    #format_fn(&self.0, f)
                }
            }

            Dummy:: #ctor_generics (#arg_n, ::std::marker::PhantomData)
        };
    )
}
