use attr;
use syn;

type Ctxt = ();

#[derive(Debug)]
pub struct Input<'a> {
    pub attrs: attr::Input,
    pub body: Body<'a>,
    pub generics: &'a syn::Generics,
    pub ident: syn::Ident,
}

#[derive(Debug)]
pub enum Body<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

#[derive(Debug)]
pub struct Variant<'a> {
    pub attrs: attr::Input,
    pub fields: Vec<Field<'a>>,
    pub ident: syn::Ident,
    pub style: Style,
}

#[derive(Debug)]
pub struct Field<'a> {
    pub attrs: attr::Field,
    pub ident: Option<syn::Ident>,
    pub ty: &'a syn::Ty,
}

#[derive(Clone, Copy, Debug)]
pub enum Style {
    Struct,
    Tuple,
    Unit,
}

impl<'a> Input<'a> {
    pub fn from_ast(cx: &Ctxt, item: &'a syn::MacroInput) -> Input<'a> {
        let attrs = attr::Input::from_ast(cx, &item.attrs);

        let body = match item.body {
            syn::Body::Enum(ref variants) => {
                Body::Enum(enum_from_ast(cx, variants))
            }
            syn::Body::Struct(ref variant_data) => {
                let (style, fields) = struct_from_ast(cx, variant_data);
                Body::Struct(style, fields)
            }
        };

        Input {
            attrs: attrs,
            body: body,
            generics: &item.generics,
            ident: item.ident.clone(),
        }
    }
}

fn enum_from_ast<'a>(cx: &Ctxt, variants: &'a [syn::Variant]) -> Vec<Variant<'a>> {
    variants
        .iter()
        .map(|variant| {
            let (style, fields) = struct_from_ast(cx, &variant.data);
            Variant {
                attrs: attr::Input::from_ast(cx, &variant.attrs),
                fields: fields,
                ident: variant.ident.clone(),
                style: style,
            }
        })
        .collect()
}

fn struct_from_ast<'a>(cx: &Ctxt, data: &'a syn::VariantData) -> (Style, Vec<Field<'a>>) {
    match *data {
        syn::VariantData::Struct(ref fields) => {
            (Style::Struct, fields_from_ast(cx, fields))
        }
        syn::VariantData::Tuple(ref fields) => {
            (Style::Tuple, fields_from_ast(cx, fields))
        }
        syn::VariantData::Unit => {
            (Style::Unit, Vec::new())
        }
    }
}

fn fields_from_ast<'a>(cx: &Ctxt, fields: &'a [syn::Field]) -> Vec<Field<'a>> {
    fields
        .iter()
        .map(|field| {
            Field {
                attrs: attr::Field::from_ast(cx, field),
                ident: field.ident.clone(),
                ty: &field.ty,
            }
        })
        .collect()
}
