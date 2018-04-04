use ast;
use attr;
use matcher;
use quote;
use syn;
use utils;

pub fn derive(input: &ast::Input) -> quote::Tokens {
    let hash_trait_path = hash_trait_path();

    let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
        .build_arms(input, |arm_path, _, _, _, bis| {
            let variant = if let ast::Data::Enum(_) = input.data {
                Some(quote! {
                    #hash_trait_path::hash(&(#arm_path as u64), __state);
                })
            } else {
                None
            };

            let field_prints = bis.iter().filter_map(|bi| {
                if bi.field.attrs.ignore_hash() {
                    return None;
                }

                let arg = bi.ident;

                if let Some(hash_with) = bi.field.attrs.hash_with() {
                    Some(quote! {
                        #hash_with(#arg, __state);
                    })
                } else {
                    Some(quote! {
                        #hash_trait_path::hash(#arg, __state);
                    })
                }
            });

            quote! {
                #variant
                #(#field_prints)*
            }
        });

    let name = input.ident;
    let hasher_trait_path = hasher_trait_path();

    let (impl_generics, path_args) = utils::build_generics(
        input,
        &hash_trait_path,
        needs_hash_bound,
        |field| field.hash_bound(),
        |input| input.hash_bound(),
    );
    let where_clause = &impl_generics.where_clause;

    let type_ = syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::PathSegment {
            ident: name,
            arguments: path_args,
        }.into(),
    });

    let hasher_type_parameter = utils::hygienic_type_parameter(input, "__H");

    quote! {
        impl #impl_generics #hash_trait_path for #type_ #where_clause {
            fn hash<#hasher_type_parameter>(&self, __state: &mut #hasher_type_parameter)
                where #hasher_type_parameter: #hasher_trait_path
            {
                match *self {
                    #body
                }
            }
        }
    }
}

fn needs_hash_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_hash() && attrs.hash_bound().is_none()
}

fn hasher_trait_path() -> syn::Path {
    parse_quote! { ::std::hash::Hasher }
}

fn hash_trait_path() -> syn::Path {
    parse_quote! { ::std::hash::Hash }
}
