use ast;
use attr;
use matcher;
use paths;
use proc_macro2::{self, TokenStream};
use syn;
use utils;

pub fn derive(input: &ast::Input) -> proc_macro2::TokenStream {
    let hasher_trait_path = hasher_trait_path();
    let hash_trait_path_input_span = hash_trait_path(input.span);

    let state = quote!(__state);

    let discriminant = if let ast::Body::Enum(_) = input.body {
        let discriminant = paths::discriminant_path();
        Some(quote!(
            #hash_trait_path_input_span::hash(&#discriminant(self), #state);
        ))
    } else {
        None
    };

    let body = matcher::Matcher::new(matcher::BindingStyle::Ref, input.attrs.is_packed).build_arms(
        input,
        "__arg",
        |_, _, _, _, _, bis| {
            let field_prints = bis.iter().filter_map(|bi| {
                if bi.field.attrs.ignore_hash() {
                    return None;
                }

                let arg = &bi.expr;

                if let Some(hash_with) = bi.field.attrs.hash_with() {
                    Some(quote! {
                        #hash_with(&#arg, #state);
                    })
                } else {
                    let hash_trait_path = hash_trait_path(bi.field.span);

                    Some(quote_spanned! {bi.field.span=>
                        #hash_trait_path::hash(&#arg, #state);
                    })
                }
            });

            quote! {
                #(#field_prints)*
            }
        },
    );

    let name = &input.ident;
    let generics = utils::build_impl_generics(
        input,
        &hash_trait_path_input_span,
        needs_hash_bound,
        |field| field.hash_bound(),
        |input| input.hash_bound(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let hasher_ty_parameter = utils::hygienic_type_parameter(input, "__H");
    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #hash_trait_path_input_span for #name #ty_generics #where_clause {
            fn hash<#hasher_ty_parameter>(&self, #state: &mut #hasher_ty_parameter)
                where #hasher_ty_parameter: #hasher_trait_path
            {
                #discriminant
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

/// Return the path of the `Hash` trait, that is `::std::hash::Hash`.
fn hash_trait_path(span: proc_macro2::Span) -> TokenStream {
    if cfg!(feature = "use_core") {
        quote_spanned!(span=> ::core::hash::Hash)
    } else {
        quote_spanned!(span=> ::std::hash::Hash)
    }
}

/// Return the path of the `Hasher` trait, that is `::std::hash::Hasher`.
fn hasher_trait_path() -> syn::Path {
    if cfg!(feature = "use_core") {
        parse_quote!(::core::hash::Hasher)
    } else {
        parse_quote!(::std::hash::Hasher)
    }
}
