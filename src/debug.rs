use quote;
use syn;
use utils;

pub fn derive(input: &syn::MacroInput, value: Option<syn::Lit>) -> quote::Tokens {
    fn make_variant_data(
        variant_name: quote::Tokens,
        variant_name_as_str: &str,
        data: &syn::VariantData,
        transparent: bool,
    ) -> quote::Tokens {
        match *data {
            syn::VariantData::Struct(ref fields) => {
                let mut field_pats = quote::Tokens::new();
                let mut field_prints = quote::Tokens::new();

                for (n, f) in fields.iter().enumerate() {
                    let name = f.ident.as_ref().unwrap();
                    field_pats.append(&format!("{}: ref __arg_{},", name, n));

                    if !utils::ignored_traits(&f.attrs).contains(&"Debug") {
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
            syn::VariantData::Tuple(_) if transparent => {
                quote!(
                    #variant_name( ref __arg_0 ) => {
                        ::std::fmt::Debug::fmt(__arg_0, f)
                    }
                )
            }
            syn::VariantData::Tuple(ref fields) => {
                let mut field_pats = quote::Tokens::new();
                let mut field_prints = quote::Tokens::new();

                for (n, f) in fields.iter().enumerate() {
                    field_pats.append(&format!("ref __arg_{},", n));

                    if !utils::ignored_traits(&f.attrs).contains(&"Debug") {
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
            syn::VariantData::Unit => {
                quote!(
                    #variant_name => f.write_str(#variant_name_as_str)
                )
            }
        }
    }

    let transparent = if let Some(syn::Lit::Str(s, _)) = value {
        s == "transparent"
    } else {
        false
    };

    let name = &input.ident;

    let arms = match input.body {
        syn::Body::Enum(ref data) => {
            let arms = data.iter().map(|field| {
                let fname = &field.ident;
                let fname_as_str = fname.as_ref();

                make_variant_data(quote!(#name::#fname), fname_as_str, &field.data, false)
            });

            quote!(#(arms),*)
        }
        syn::Body::Struct(ref vd) => {
            let arms = make_variant_data(quote!(#name), name.as_ref(), vd, transparent);

            quote!(#arms)
        }
    };

    let debug_trait_path = syn::parse_path("::std::fmt::Debug").unwrap();
    let impl_generics = syn::aster::from_generics(input.generics.clone())
                                  .add_ty_param_bound(debug_trait_path.clone())
                                  .build();
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
