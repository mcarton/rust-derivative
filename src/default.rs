use ast;
use attr;
use bound;
use quote;
use syn::{self, aster};
use utils;

pub fn derive(input: &ast::Input) -> quote::Tokens {
    fn make_variant_data(
        variant_name: quote::Tokens,
        style: ast::Style,
        fields: &[ast::Field],
    ) -> quote::Tokens {
        match style {
            ast::Style::Struct => {
                let mut defaults = Vec::new();

                for f in fields {
                    let name = f.ident.as_ref().unwrap();
                    let default = f.attrs.default_value().map_or_else(
                        || quote!(::std::default::Default::default()),
                        |v| quote!(#v),
                    );

                    defaults.push(quote!(#name: #default));
                }

                quote!(#variant_name { #(defaults),* })
            }
            ast::Style::Tuple => {
                let mut defaults = Vec::new();

                for f in fields {
                    let default = f.attrs.default_value().map_or_else(
                        || quote!(::std::default::Default::default()),
                        |v| quote!(#v),
                    );

                    defaults.push(default);
                }

                quote!(#variant_name ( #(defaults),* ))
            }
            ast::Style::Unit => quote!(#variant_name),
        }
    }

    let name = &input.ident;
    let default_trait_path = default_trait_path();
    let impl_generics = utils::build_impl_generics(
        input,
        &default_trait_path,
        needs_default,
        |_| Some(&[]),
        |_| Some(&[]),
    );
    let where_clause = &impl_generics.where_clause;

    let ty = syn::aster::ty().path()
                             .segment(name.clone())
                             .with_generics(impl_generics.clone())
                             .build()
                             .build();

    let body = match input.body {
        ast::Body::Enum(ref data) => {
            let arms = data.iter().filter_map(|variant| {
                if variant.attrs.default {
                    let vname = &variant.ident;
                    let vname_as_str = vname.as_ref();

                    Some(make_variant_data(quote!(#name::#vname), variant.style, &variant.fields))
                } else {
                    None
                }
            });

            quote!(#(arms),*)
        }
        ast::Body::Struct(style, ref vd) => {
            make_variant_data(quote!(#name), style, vd)
        }
    };

    quote!(
        impl #impl_generics #default_trait_path for #ty #where_clause {
            fn default() -> Self {
                #body
            }
        }
    )
}

/// Return the path of the `Default` trait, that is `::std::default::Default`.
fn default_trait_path() -> syn::Path {
    aster::path().global().ids(&["std", "default", "Default"]).build()
}

fn needs_default(attrs: &attr::Field) -> bool {
    true
}
