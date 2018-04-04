use ast;
use attr;
use matcher;
use quote;
use syn;
use utils;

pub fn derive_copy(input: &ast::Input) -> Result<quote::Tokens, String> {
    if input.attrs.derives_clone() {
        return Err("`#[derivative(Copy)]` can't be used with `#[derive(Clone)]`".to_string());
    }

    let name = input.ident;
    let copy_trait_path = copy_trait_path();

    let (impl_generics, path_args) = utils::build_generics(
        input,
        &copy_trait_path,
        |attrs| attrs.copy_bound().is_none(),
        |field| field.copy_bound(),
        |input| input.copy_bound(),
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
        impl #impl_generics #copy_trait_path for #type_ #where_clause {}
    })
}

/// Derive `Clone` for `input`.
pub fn derive_clone(input: &ast::Input) -> quote::Tokens {
    let name = input.ident;
    let clone_trait_path = clone_trait_path();

    let (impl_generics, path_args) = utils::build_generics(
        input,
        &clone_trait_path,
        needs_clone_bound,
        |field| field.clone_bound(),
        |input| input.clone_bound(),
    );
    let where_clause = &impl_generics.where_clause;

    let type_ = syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::PathSegment {
            ident: name,
            arguments: path_args,
        }.into(),
    });

    let is_copy = input.attrs.rustc_copy_clone_marker() || input.attrs.copy.is_some();
    if is_copy && input.generics.type_params().count() == 0 {
        quote! {
            impl #impl_generics #clone_trait_path for #type_ #where_clause {
                fn clone(&self) -> Self {
                    *self
                }
            }
        }
    } else {
        let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
            .build_arms(input, |arm_path, _, style, _, bis| {
                let field_clones = bis.iter().map(|bi| {
                    let arg = bi.ident;

                    let clone = if let Some(clone_with) = bi.field.attrs.clone_with() {
                        quote!(#clone_with(#arg))
                    } else {
                        quote!(#arg.clone())
                    };

                    if let Some(ref name) = bi.field.ident {
                        quote! {
                            #name: #clone
                        }
                    } else {
                        clone
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
            });

        let clone_from_body = if input.attrs.clone_from() {
            let clone_from_def = matcher::Matcher::new(matcher::BindingStyle::RefMut)
                .build_arms(input, |outer_arm_path, _, _, _, outer_bis| {
                    let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
                        .with_name("__other".into())
                        .build_arms(input, |inner_arm_path, _, _, _, inner_bis| {
                            if outer_arm_path == inner_arm_path {
                                let field_clones = outer_bis
                                    .iter()
                                    .zip(inner_bis)
                                    .map(|(outer_bi, inner_bi)| {
                                        let outer = outer_bi.ident;
                                        let inner = inner_bi.ident;

                                        quote!(#outer.clone_from(#inner);)
                                    });

                                quote! {
                                    #(#field_clones)*
                                    return;
                                }
                            } else {
                                quote!()
                            }
                        });

                    quote! {
                        match *other {
                            #body
                        }
                    }
                });

            Some(clone_from_def)
        } else {
            None
        };

        let clone_from_item = clone_from_body.map(|body| {
            // Enumerations are only cloned-from if both variants are the same.
            // If they are different, fallback to normal cloning.
            let fallback = if let ast::Data::Enum(_) = input.data {
                Some(quote!(*self = other.clone();))
            } else {
                None
            };

            quote! {
                fn clone_from(&mut self, other: &Self) {
                    match *self {
                        #body
                    }

                    #fallback
                }
            }
        });

        quote! {
            impl #impl_generics #clone_trait_path for #type_ #where_clause {
                fn clone(&self) -> Self {
                    match *self {
                        #body
                    }
                }

                #clone_from_item
            }
        }
    }
}

fn needs_clone_bound(attrs: &attr::Field) -> bool {
    attrs.clone_bound().is_none()
}

/// Return the path of the `Copy` trait, that is `::std::marker::Copy`.
fn copy_trait_path() -> syn::Path {
    parse_quote! { ::std::marker::Copy }
}

/// Return the path of the `Clone` trait, that is `::std::clone::Clone`.
fn clone_trait_path() -> syn::Path {
    parse_quote! { ::std::clone::Clone }
}
