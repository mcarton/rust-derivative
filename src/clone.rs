use ast;
use attr;
use matcher;
use quote;
use syn::{self, aster};
use utils;

/// Derive `Copy` for `input`.
pub fn derive_copy(input: &ast::Input) -> quote::Tokens {
    let name = &input.ident;

    let copy_trait_path = copy_trait_path();
    let impl_generics = utils::build_impl_generics(
        input,
        &copy_trait_path,
        |attrs| attrs.copy_bound().is_none(),
        |field| field.copy_bound(),
        |input| input.copy_bound(),
    );
    let where_clause = &impl_generics.where_clause;

    let ty = syn::aster::ty().path()
                             .segment(name.clone())
                             .with_generics(impl_generics.clone())
                             .build()
                             .build();

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #copy_trait_path for #ty #where_clause {}
    }
}

/// Derive `Clone` for `input`.
pub fn derive_clone(input: &ast::Input) -> quote::Tokens {
    let name = &input.ident;

    let clone_trait_path = clone_trait_path();
    let impl_generics = utils::build_impl_generics(
        input,
        &clone_trait_path,
        needs_clone_bound,
        |field| field.clone_bound(),
        |input| input.clone_bound(),
    );
    let where_clause = &impl_generics.where_clause;

    let ty = syn::aster::ty().path()
                             .segment(name.clone())
                             .with_generics(impl_generics.clone())
                             .build()
                             .build();

    if input.attrs.copy.is_some() && input.generics.ty_params.is_empty() {
        quote! {
            #[allow(unused_qualifications)]
            impl #impl_generics #clone_trait_path for #ty #where_clause {
                fn clone(&self) -> Self {
                    *self
                }
            }
        }
    } else {
        let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
            .build_arms(input, |arm_path, _, style, _, bis| {
                let field_clones = bis.iter().map(|bi| {
                    let arg = &bi.ident;

                    if let Some(ref name) = bi.field.ident {
                        quote! {
                            #name: #arg.clone()
                        }
                    } else {
                        quote! {
                            #arg.clone()
                        }
                    }
                });

                match style {
                    ast::Style::Struct => {
                        quote! {
                            #arm_path {
                                #(#field_clones),*
                            }
                        }
                    }
                    ast::Style::Tuple => {
                        quote! {
                            #arm_path (#(#field_clones),*)
                        }
                    }
                    ast::Style::Unit => {
                        quote! {
                            #arm_path
                        }
                    }
                }
            }
        );

        quote! {
            #[allow(unused_qualifications)]
            impl #impl_generics #clone_trait_path for #ty #where_clause {
                fn clone(&self) -> Self {
                    match *self {
                        #body
                    }
                }
            }
        }
    }
}

fn needs_clone_bound(attrs: &attr::Field) -> bool {
    attrs.clone_bound().is_none()
}

/// Return the path of the `Clone` trait, that is `::std::clone::Clone`.
fn clone_trait_path() -> syn::Path {
    aster::path().global().ids(&["std", "clone", "Clone"]).build()
}

/// Return the path of the `Copy` trait, that is `::std::marker::Clone`.
fn copy_trait_path() -> syn::Path {
    aster::path().global().ids(&["std", "marker", "Copy"]).build()
}
