use ast;
use attr;
use matcher;
use quote;
use syn;
use utils;

pub fn derive(input: &ast::Input) -> quote::Tokens {
    let body = matcher::Matcher::new(matcher::BindingStyle::Ref)
        .build_arms(input, |_, arm_name, style, attrs, bis| {
            let field_prints = bis.iter().filter_map(|bi| {
                if bi.field.attrs.ignore_debug() {
                    return None;
                }

                if attrs.debug_transparent() {
                    return Some(quote!{
                        ::std::fmt::Debug::fmt(__arg_0, __f)
                    });
                }

                let arg = bi.ident;

                let dummy_debug = bi.field.attrs.debug_format_with().map(|format_fn| {
                    format_with(bi.field, &arg, format_fn, input.generics.clone())
                });

                let builder = if let Some(ref name) = bi.field.ident {
                    let name = name.as_ref();
                    quote! {
                        #dummy_debug
                        let _ = builder.field(#name, &#arg);
                    }
                } else {
                    quote! {
                        #dummy_debug
                        let _ = builder.field(&#arg);
                    }
                };

                Some(builder)
            });

            let method = match style {
                ast::Style::Struct => "debug_struct",
                ast::Style::Tuple | ast::Style::Unit => "debug_tuple",
            };
            let method = syn::Ident::from(method);

            let name = arm_name.as_ref();

            if attrs.debug_transparent() {
                quote! {
                    #(#field_prints)*
                }
            } else {
                quote! {
                    let mut builder = __f.#method(#name);
                    #(#field_prints)*
                    builder.finish()
                }
            }
        });

    let name = input.ident;
    let debug_trait_path = debug_trait_path();

    let (impl_generics, path_args) = utils::build_generics(
        input,
        &debug_trait_path,
        needs_debug_bound,
        |field| field.debug_bound(),
        |input| input.debug_bound(),
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
        impl #impl_generics #debug_trait_path for #type_ #where_clause {
            fn fmt(&self, __f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self {
                    #body
                }
            }
        }
    }
}

fn needs_debug_bound(attrs: &attr::Field) -> bool {
    !attrs.ignore_debug() && attrs.debug_bound().is_none()
}

/// Return the path of the `Debug` trait, that is `::std::fmt::Debug`.
fn debug_trait_path() -> syn::Path {
    parse_quote! { ::std::fmt::Debug }
}

fn format_with(
    f: &ast::Field,
    arg_n: &syn::Ident,
    format_fn: &syn::Path,
    mut generics: syn::Generics,
) -> quote::Tokens {
    let debug_trait_path = debug_trait_path();

    let ctor_generics = generics.clone();
    let (_, ctor_type_generics, _) = ctor_generics.split_for_impl();

    fn new_where_clause() -> syn::WhereClause {
        syn::WhereClause {
            where_token: <Token![where]>::default(),
            predicates: syn::punctuated::Punctuated::new(),
        }
    }

    fn get_or_insert_with<F, T>(option: &mut Option<T>, f: F) -> &mut T
        where F: FnOnce() -> T,
    {
        match *option {
            None => *option = Some(f()),
            _ => (),
        }

        match *option {
            Some(ref mut v) => v,
            _ => unreachable!(),
        }
    }

    generics
        .make_where_clause()
        .predicates
        .extend(f.attrs
            .debug_bound()
            .unwrap_or(&[])
            .iter()
            .cloned());
    generics.params.push(parse_quote!('_derivative));

    for type_param in generics.params.iter().filter_map(|param| {
        match *param {
            syn::GenericParam::Type(ref type_param) => Some(type_param),
            _ => None,
        }
    }) {
        let path = type_param.ident.into();
        let bound: syn::TypeParamBound = parse_quote!('_derivative);

        get_or_insert_with(&mut generics.where_clause, new_where_clause)
            .predicates
            .push(syn::WherePredicate::Type(syn::PredicateType {
                lifetimes: None,
                bounded_ty: syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: path,
                }),
                colon_token: <Token![:]>::default(),
                bounds: vec![bound].into_iter().collect(),
            }));
    }

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let type_ = f.type_;

    // Leave off the type parameter bounds, defaults, and attributes
    let phantom = generics.type_params().map(|tp| tp.ident);

    quote! {
        let #arg_n = {
            struct Dummy #type_generics (&'_derivative #type_, ::std::marker::PhantomData <(#(#phantom),*)>) #where_clause;

            impl #impl_generics #debug_trait_path for Dummy #type_generics #where_clause {
                fn fmt(&self, __f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    #format_fn(&self.0, __f)
                }
            }

            Dummy:: #ctor_type_generics (#arg_n, ::std::marker::PhantomData)
        };
    }
}
















