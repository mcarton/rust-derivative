/* This file incorporates work covered by the following copyright and
 * permission notice:
 *   Copyright 2016 The serde Developers. See
 *   https://github.com/serde-rs/serde/blob/3f28a9324042950afa80354722aeeee1a55cbfa3/README.md#license.
 *
 *   Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 *   http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 *   <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 *   option. This file may not be copied, modified, or distributed
 *   except according to those terms.
 */

use ast;
use attr;
use std::collections::HashSet;
use syn;

/// Remove the default from every type parameter because in the generated `impl`s
/// they look like associated types: "error: associated type bindings are not
/// allowed here".
pub fn without_defaults(generics: &syn::Generics) -> syn::Generics {
    syn::Generics {
        params: generics.params
            .iter()
            .map(|param| {
                match *param {
                    syn::GenericParam::Type(ref type_param) => {
                        syn::GenericParam::Type(syn::TypeParam {
                            default: None,
                            ..type_param.clone()
                        })
                    }
                    ref other => other.clone(),
                }
            })
            .collect(),
        ..generics.clone()
    }
}

pub fn with_where_predicates(
    generics: &syn::Generics,
    predicates: &[syn::WherePredicate],
) -> syn::Generics {
    let predicates = predicates.iter().cloned();

    syn::Generics {
        where_clause: Some(
            match generics.where_clause {
                Some(ref where_clause) => {
                    syn::WhereClause {
                        where_token: where_clause.where_token,
                        predicates: where_clause.predicates.iter().cloned().chain(predicates).collect(),
                    }
                }
                None => {
                    syn::WhereClause {
                        where_token: <Token![where]>::default(),
                        predicates: predicates.collect(),
                    }
                }
            }
        ),
        ..generics.clone()
    }
}

pub fn with_where_predicates_from_fields<F>(
    item: &ast::Input,
    generics: &syn::Generics,
    from_field: F,
) -> syn::Generics
    where F: Fn(&attr::Field) -> Option<&[syn::WherePredicate]>,
{
    let all_fields = item.data.all_fields();
    let predicates = all_fields
        .iter()
        .flat_map(|field| from_field(&field.attrs))
        .flat_map(|predicates| predicates.iter())
        .cloned();

    syn::Generics {
        where_clause: Some(
            match generics.where_clause {
                Some(ref where_clause) => {
                    syn::WhereClause {
                        where_token: where_clause.where_token,
                        predicates: where_clause.predicates.iter().cloned().chain(predicates).collect(),
                    }
                }
                None => {
                    syn::WhereClause {
                        where_token: <Token![where]>::default(),
                        predicates: predicates.collect(),
                    }
                }
            }
        ),
        ..generics.clone()
    }
}

/// Puts the given bound on any generic type parameters that are used in fields
/// for which filter returns true.
///
/// For example, the following structure needs the bound `A: Debug, B: Debug`.
///
/// ```ignore
/// struct S<'b, A, B: 'b, C> {
///     a: A,
///     b: Option<&'b B>
///     #[derivative(Debug="ignore")]
///     c: C,
/// }
/// ```
pub fn with_bound<F>(
    item: &ast::Input,
    generics: &syn::Generics,
    filter: F,
    bound: &syn::Path,
) -> syn::Generics
    where F: Fn(&attr::Field) -> bool,
{
    #[derive(Debug)]
    struct FindTypeParams {
        /// Set of all generic type parameters on the current struct (A, B, C in
        /// the example). Initialized up front.
        all_type_params: HashSet<syn::Ident>,
        /// Set of generic type parameters used in fields for which filter
        /// returns true (A and B in the example). Filled in as the visitor sees
        /// them.
        relevant_type_params: HashSet<syn::Ident>,
    }

    impl<'ast> syn::visit::Visit<'ast> for FindTypeParams {
        fn visit_path(&mut self, path: &'ast syn::Path) {
            if let Some(seg) = path.segments.last().map(|pair| pair.into_value()) {
                if seg.ident == "PhantomData" {
                    // Hardcoded exception, because `PhantomData<T>` implements
                    // most traits whether or not `T` implements it.
                    return;
                }
            }

            if !path.global() && path.segments.len() == 1 {
                let id = path.segments[0].ident;
                if self.all_type_params.contains(&id) {
                    self.relevant_type_params.insert(id);
                }
            }

            syn::visit::visit_path(self, path);
        }
    }

    let all_type_params: HashSet<_> = generics.type_params()
        .map(|type_param| type_param.ident)
        .collect();

    let relevant_types = item.data
        .all_fields()
        .into_iter()
        .filter(|field| filter(&field.attrs))
        .map(|field| &field.type_);

    let mut visitor = FindTypeParams {
        all_type_params: all_type_params,
        relevant_type_params: HashSet::new(),
    };

    for type_ in relevant_types {
        syn::visit::visit_type(&mut visitor, type_);
    }

    let predicates = generics.type_params()
        .map(|type_param| type_param.ident)
        .filter(|id| visitor.relevant_type_params.contains(id))
        .map(|id| {
            parse_quote! {
                #id: #bound
            }
        });

    syn::Generics {
        where_clause: Some(
            match generics.where_clause {
                Some(ref where_clause) => {
                    syn::WhereClause {
                        where_token: where_clause.where_token,
                        predicates: where_clause.predicates.iter().cloned().chain(predicates).collect(),
                    }
                }
                None => {
                    syn::WhereClause {
                        where_token: <Token![where]>::default(),
                        predicates: predicates.collect(),
                    }
                }
            }
        ),
        ..generics.clone()
    }
}
