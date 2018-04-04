#![allow(dead_code)] // TODO: remove

// This is inspired from `synstructure`, but `synstructure` is not adapted in severals ways
// including:
//     * `&mut` everywhere
//     * not generic, we use our own `ast`, `synstructure` only knows about `syn`
//     * missing information (what arm are we in?, what attributes? etc.)

use ast;
use attr;
use quote::{self, ToTokens};
use syn;

/// The type of binding to use when generating a pattern.
#[derive(Debug, Copy, Clone)]
pub enum BindingStyle {
    /// `x`
    Move,
    /// `mut x`
    MoveMut,
    /// `ref x`
    Ref,
    /// `ref mut x`
    RefMut,
}

fn to_pat_ident(binding_style: BindingStyle, ident: syn::Ident) -> syn::PatIdent {
    let (by_ref, mutability) = match binding_style {
        BindingStyle::Move    => (None,                           None),
        BindingStyle::MoveMut => (None,                           Some(<Token![mut]>::default())),
        BindingStyle::Ref     => (Some(<Token![ref]>::default()), None),
        BindingStyle::RefMut  => (Some(<Token![ref]>::default()), Some(<Token![mut]>::default())),
    };

    syn::PatIdent {
        by_ref: by_ref,
        mutability: mutability,
        ident: ident,
        subpat: None,
    }
}

#[derive(Debug)]
pub struct BindingInfo<'a> {
    pub ident: syn::Ident,
    pub field: &'a ast::Field<'a>,
}

pub struct Matcher {
    binding_name: String,
    binding_style: BindingStyle,
}

impl Matcher {
    pub fn new(style: BindingStyle) -> Self {
        Matcher {
            binding_name: "__arg".into(),
            binding_style: style,
        }
    }

    pub fn with_name(self, name: String) -> Self {
        Matcher { binding_name: name, ..self }
    }

    pub fn build_arms<F>(self, input: &ast::Input, f: F) -> quote::Tokens
        where F: Fn(syn::Path, &syn::Ident, ast::Style, &attr::Input, Vec<BindingInfo>) -> quote::Tokens
    {
        let ident = input.ident;

        // Generate patterns for matching against all of the variants
        let variants = match input.data {
            ast::Data::Enum(ref variants) => {
                variants
                    .iter()
                    .map(|variant| {
                        let variant_ident = variant.ident;
                        let variant_path: syn::Path = parse_quote! { #ident::#variant_ident };

                        let pattern_info = self.build_match_pattern(
                            variant_path.clone(),
                            variant.style,
                            &variant.fields,
                        );

                        (variant_path, variant_ident, variant.style, &variant.attrs, pattern_info)
                    })
                    .collect()
            }
            ast::Data::Struct(style, ref fields) => {
                let path = ident.into();
                vec![(path, ident, style, &input.attrs, self.build_match_pattern(ident, style, fields))]
            }
        };

        // Now that we have the patterns, generate the actual branches of the match
        // expression
        let processed = variants
            .into_iter()
            .map(|(path, name, style, attrs, (pattern, bindings))| {
                let body = f(path, &name, style, attrs, bindings);
                quote! { #pattern => { #body } }
            });

        quote! {
            #(#processed)*
        }
    }

    pub fn build_match_pattern<'a, N>(
        &self,
        name: N,
        style: ast::Style,
        fields: &'a [ast::Field<'a>]
    ) -> (quote::Tokens, Vec<BindingInfo<'a>>)
        where N: Into<syn::Path>,
    {
        let matches: Vec<_> = fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                BindingInfo {
                    ident: format!("{}_{}", self.binding_name, i).into(),
                    field: field,
                }
            })
            .collect();

        let match_expr = match style {
            ast::Style::Unit => syn::Pat::Path(syn::PatPath {
                qself: None,
                path: name.into(),
            }),
            ast::Style::Tuple => syn::Pat::TupleStruct(syn::PatTupleStruct {
                path: name.into(),
                pat: syn::PatTuple {
                    paren_token: syn::token::Paren::default(),
                    front: matches
                        .iter()
                        .map(|&BindingInfo { ref ident, .. }| {
                            syn::Pat::Ident(to_pat_ident(self.binding_style, *ident))
                        })
                        .collect(),
                    dot2_token: None,
                    comma_token: None,
                    back: syn::punctuated::Punctuated::new(),
                },
            }),
            ast::Style::Struct => syn::Pat::Struct(syn::PatStruct {
                path: name.into(),
                brace_token: syn::token::Brace::default(),
                fields: matches
                    .iter()
                    .map(|&BindingInfo { ref ident, ref field }| {
                        syn::FieldPat {
                            attrs: vec![],
                            member: syn::Member::Named(field.ident.unwrap()),
                            colon_token: Some(<Token![:]>::default()),
                            pat: Box::new(syn::Pat::Ident(to_pat_ident(self.binding_style, *ident))),
                        }
                    })
                    .collect(),
                dot2_token: None,
            }),
        };

        (match_expr.into_tokens(), matches)
    }
}
