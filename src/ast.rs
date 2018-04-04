use attr;
use syn;

#[derive(Debug)]
pub struct Input<'a> {
    pub attrs: attr::Input,
    pub data: Data<'a>,
    pub generics: &'a syn::Generics,
    pub ident: syn::Ident,
}

#[derive(Debug)]
pub enum Data<'a> {
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
    pub type_: &'a syn::Type,
}

#[derive(Clone, Copy, Debug)]
pub enum Style {
    Struct,
    Tuple,
    Unit,
}

impl<'a> Input<'a> {
    pub fn from_ast(item: &'a syn::DeriveInput) -> Result<Input<'a>, String> {
        let attrs = attr::Input::from_ast(&item.attrs)?;

        let data = match item.data {
            syn::Data::Enum(ref data_enum) => {
                Data::Enum(enum_from_ast(&data_enum.variants)?)
            }
            syn::Data::Struct(ref data_struct) => {
                let (style, fields) = struct_like_data_from_ast(&data_struct.fields)?;
                Data::Struct(style, fields)
            }
            syn::Data::Union(_) => {
                // TODO: support or not support?
                return Err("`derivative` doesn't support unions".to_string());
            }
        };

        Ok(Input {
            attrs: attrs,
            data: data,
            generics: &item.generics,
            ident: item.ident,
        })
    }
}

impl<'a> Data<'a> {
    pub fn all_fields(&self) -> Vec<&Field> {
        match *self {
            Data::Enum(ref variants) => {
                variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter())
                    .collect()
            }
            Data::Struct(_, ref fields) => fields.iter().collect(),
        }
    }
}

fn enum_from_ast<'a>(
    variants: &'a syn::punctuated::Punctuated<syn::Variant, Token![,]>,
) -> Result<Vec<Variant<'a>>, String> {
    variants
        .iter()
        .map(|variant| {
            let (style, fields) = struct_like_data_from_ast(&variant.fields)?;
            Ok(Variant {
                attrs: attr::Input::from_ast(&variant.attrs)?,
                fields: fields,
                ident: variant.ident,
                style: style,
            })
        })
        .collect()
}

fn struct_like_data_from_ast<'a>(fields: &'a syn::Fields) -> Result<(Style, Vec<Field<'a>>), String> {
    match *fields {
        syn::Fields::Named(syn::FieldsNamed { ref named, .. }) => {
            Ok((Style::Struct, fields_from_ast(named)?))
        }
        syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => {
            Ok((Style::Tuple, fields_from_ast(unnamed)?))
        }
        syn::Fields::Unit => {
            Ok((Style::Unit, Vec::new()))
        }
    }
}

fn fields_from_ast<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, Token![,]>,
) -> Result<Vec<Field<'a>>, String> {
    fields
        .iter()
        .map(|field| {
            Ok(Field {
                attrs: attr::Field::from_ast(field)?,
                ident: field.ident,
                type_: &field.ty,
            })
        })
        .collect()
}
