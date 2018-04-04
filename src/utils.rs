use ast;
use attr;
use bound;
use syn;

/// Make generic with all the generics in the input, plus a bound `T: <trait_path>` for each
/// generic field type that will be shown.
pub fn build_generics<F, G, H>(
    item: &ast::Input,
    trait_path: &syn::Path,
    needs_debug_bound: F,
    field_bound: G,
    input_bound: H,
) -> (syn::Generics, syn::PathArguments)
    where F: Fn(&attr::Field) -> bool,
          G: Fn(&attr::Field) -> Option<&[syn::WherePredicate]>,
          H: Fn(&attr::Input) -> Option<&[syn::WherePredicate]>,
{
    let generics = bound::without_defaults(item.generics);
    let generics = bound::with_where_predicates_from_fields(
        item, &generics, field_bound
    );

    let impl_generics = match input_bound(&item.attrs) {
        Some(predicates) => {
            bound::with_where_predicates(&generics, predicates)
        }
        None => {
            bound::with_bound(item, &generics, needs_debug_bound, trait_path)
        }
    };

    let path_args = if impl_generics.params.is_empty() {
        syn::PathArguments::None
    } else {
        let generic_args = impl_generics.params
            .iter()
            .map(|param| {
                match *param {
                    syn::GenericParam::Type(syn::TypeParam { ident, .. }) => {
                        syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: ident.into(),
                        }))
                    }
                    syn::GenericParam::Lifetime(syn::LifetimeDef { lifetime, .. }) => {
                        syn::GenericArgument::Lifetime(lifetime)
                    }
                    syn::GenericParam::Const(_) => {
                        // TODO: yet?
                        unimplemented!();
                    }
                }
            })
            .collect();

        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: <Token![<]>::default(),
            args: generic_args,
            gt_token: <Token![>]>::default(),
        })
    };

    (impl_generics, path_args)
}

// TODO: this function should be obsolete with `Span::def_site()` when it stabilizes
/// Construct a name for the inner type parameter that can't collide with any
/// type parameters of the item. This is achieved by starting with a base and
/// then concatenating the names of all other type parameters.
pub fn hygienic_type_parameter(item: &ast::Input, base: &str) -> syn::Ident {
    let mut string_type_param = base.to_string();

    for param in item.generics.type_params() {
        string_type_param.push_str(param.ident.as_ref());
    }

    syn::Ident::from(string_type_param)
}
