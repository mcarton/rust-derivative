#![allow(dead_code)] // TODO: remove

// This is inspired from `synstructure`, but `synstructure` is not adapted in severals ways
// including:
//     * `&mut` everywhere
//     * not generic, we use our own `ast`, `synstructure` only knows about `syn`
//     * missing information (what arm are we in?, what attributes? etc.)

use proc_macro2::{self, TokenStream};
use quote::ToTokens;
use syn;

use ast;
use attr;
use quote;

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

impl quote::ToTokens for BindingStyle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match *self {
            BindingStyle::Move => (),
            BindingStyle::MoveMut => tokens.extend(quote!(mut)),
            BindingStyle::Ref => tokens.extend(quote!(ref)),
            BindingStyle::RefMut => {
                tokens.extend(quote!(ref mut));
            }
        }
    }
}

#[derive(Debug)]
pub struct BindingInfo<'a> {
    pub ident: syn::Ident,
    pub field: &'a ast::Field<'a>,
}

#[derive(Debug)]
pub struct CommonVariant<'a> {
    path: syn::Path,
    name: &'a syn::Ident,
    style: ast::Style,
    attrs: &'a attr::Input,
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
        Matcher {
            binding_name: name,
            ..self
        }
    }

    pub fn build_arms<F>(self, input: &ast::Input, binding_name: &str, f: F) -> TokenStream
    where
        F: Fn(
            syn::Path,
            usize,
            &syn::Ident,
            ast::Style,
            &attr::Input,
            Vec<BindingInfo>,
        ) -> TokenStream,
    {
        let variants = self.build_match_pattern(input, binding_name);

        // Now that we have the patterns, generate the actual branches of the match
        // expression
        let mut t = TokenStream::new();
        for (i, (variant, (pat, bindings))) in variants.into_iter().enumerate() {
            let body = f(
                variant.path,
                i,
                variant.name,
                variant.style,
                variant.attrs,
                bindings,
            );
            quote!(#pat => { #body }).to_tokens(&mut t);
        }

        t
    }

    pub fn build_2_arms<F>(
        self,
        left: (&ast::Input, &str),
        right: (&ast::Input, &str),
        f: F,
    ) -> TokenStream
    where
        F: Fn(
            usize,
            CommonVariant,
            CommonVariant,
            (Vec<BindingInfo>, Vec<BindingInfo>),
        ) -> TokenStream,
    {
        let left_variants = self.build_match_pattern(left.0, left.1);
        let right_variants = self.build_match_pattern(right.0, right.1);

        assert_eq!(left_variants.len(), right_variants.len());

        // Now that we have the patterns, generate the actual branches of the match
        // expression
        let mut t = TokenStream::new();
        for (i, (left, right)) in left_variants.into_iter().zip(right_variants).enumerate() {
            let (left, (left_pat, left_bindings)) = left;
            let (right, (right_pat, right_bindings)) = right;

            let body = f(i, left, right, (left_bindings, right_bindings));
            quote!((#left_pat, #right_pat) => { #body }).to_tokens(&mut t);
        }

        t
    }

    /// Generate patterns for matching against all of the variants
    pub fn build_match_pattern<'a>(
        &self,
        input: &'a ast::Input,
        binding_name: &str,
    ) -> Vec<(CommonVariant<'a>, (TokenStream, Vec<BindingInfo<'a>>))> {
        let ident = &input.ident;

        match input.body {
            ast::Body::Enum(ref variants) => variants
                .iter()
                .map(|variant| {
                    let variant_ident = &variant.ident;
                    let path = parse_quote!(#ident::#variant_ident);

                    let pat = self.build_match_pattern_impl(
                        &path,
                        variant.style,
                        &variant.fields,
                        binding_name,
                    );

                    (
                        CommonVariant {
                            path,
                            name: variant_ident,
                            style: variant.style,
                            attrs: &variant.attrs,
                        },
                        pat,
                    )
                })
                .collect(),
            ast::Body::Struct(style, ref vd) => {
                let path = parse_quote!(#ident);
                vec![(
                    CommonVariant {
                        path,
                        name: ident,
                        style,
                        attrs: &input.attrs,
                    },
                    self.build_match_pattern_impl(ident, style, vd, binding_name),
                )]
            }
        }
    }

    fn build_match_pattern_impl<'a, N>(
        &self,
        name: &N,
        style: ast::Style,
        fields: &'a [ast::Field<'a>],
        binding_name: &str,
    ) -> (TokenStream, Vec<BindingInfo<'a>>)
    where
        N: quote::ToTokens,
    {
        let (stream, matches) = match style {
            ast::Style::Unit => (TokenStream::new(), Vec::new()),
            ast::Style::Tuple => {
                let (stream, matches) = fields.iter().enumerate().fold(
                    (TokenStream::new(), Vec::new()),
                    |(stream, matches), field| {
                        self.build_inner_pattern(
                            (stream, matches),
                            field,
                            binding_name,
                            |_, ident, binding| quote!(#binding #ident ,),
                        )
                    },
                );

                (quote! { ( #stream ) }, matches)
            }
            ast::Style::Struct => {
                let (stream, matches) = fields.iter().enumerate().fold(
                    (TokenStream::new(), Vec::new()),
                    |(stream, matches), field| {
                        self.build_inner_pattern(
                            (stream, matches),
                            field,
                            binding_name,
                            |field, ident, binding| {
                                let field_name = field.ident.as_ref().unwrap();
                                quote!(#field_name : #binding #ident ,)
                            },
                        )
                    },
                );

                (quote! { { #stream } }, matches)
            }
        };

        let mut all_tokens = TokenStream::new();
        name.to_tokens(&mut all_tokens);
        all_tokens.extend(stream);

        (all_tokens, matches)
    }

    fn build_inner_pattern<'a>(
        &self,
        (mut stream, mut matches): (TokenStream, Vec<BindingInfo<'a>>),
        (i, field): (usize, &'a ast::Field),
        binding_name: &str,
        f: impl FnOnce(&ast::Field, &syn::Ident, BindingStyle) -> TokenStream,
    ) -> (TokenStream, Vec<BindingInfo<'a>>) {
        let binding_style = self.binding_style;

        let ident: syn::Ident = syn::Ident::new(
            &format!("{}_{}", binding_name, i),
            proc_macro2::Span::call_site(),
        );

        f(field, &ident, binding_style).to_tokens(&mut stream);

        matches.push(BindingInfo {
            ident,
            field,
        });

        (stream, matches)
    }
}
