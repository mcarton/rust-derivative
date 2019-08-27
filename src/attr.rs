use proc_macro2;
use syn;

/// Represent the `derivative` attributes on the input type (`struct`/`enum`).
#[derive(Debug, Default)]
pub struct Input {
    /// Whether `Clone` is present and its specific attributes.
    pub clone: Option<InputClone>,
    /// Whether `Copy` is present and its specific attributes.
    pub copy: Option<InputCopy>,
    /// Whether `Debug` is present and its specific attributes.
    pub debug: Option<InputDebug>,
    /// Whether `Default` is present and its specific attributes.
    pub default: Option<InputDefault>,
    /// Whether `Eq` is present and its specific attributes.
    pub eq: Option<InputEq>,
    /// Whether `Hash` is present and its specific attributes.
    pub hash: Option<InputHash>,
    /// Whether `Eq` is present and its specific attributes.
    pub partial_eq: Option<InputPartialEq>,
}

#[derive(Debug, Default)]
/// Represent the `derivative` attributes on a field.
pub struct Field {
    /// The parameters for `Clone`.
    clone: FieldClone,
    /// The parameters for `Copy`.
    copy_bound: Option<Vec<syn::WherePredicate>>,
    /// The parameters for `Debug`.
    debug: FieldDebug,
    /// The parameters for `Default`.
    default: FieldDefault,
    /// The parameters for `Eq`.
    eq_bound: Option<Vec<syn::WherePredicate>>,
    /// The parameters for `Hash`.
    hash: FieldHash,
    /// The parameters for `Eq`.
    partial_eq: FieldPartialEq,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Clone(…))` attributes on an input.
pub struct InputClone {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Whether the implementation should have an explicit `clone_from`.
    pub clone_from: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Clone(…))` attributes on an input.
pub struct InputCopy {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Debug(…))` attributes on an input.
pub struct InputDebug {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Whether the type is marked `transparent`.
    pub transparent: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Default(…))` attributes on an input.
pub struct InputDefault {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Whether the type is marked with `new`.
    pub new: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Eq(…))` attributes on an input.
pub struct InputEq {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Hash(…))` attributes on an input.
pub struct InputHash {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
}

#[derive(Debug, Default)]
/// Represent the `derivative(PartialEq(…))` attributes on an input.
pub struct InputPartialEq {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Allow `derivative(PartialEq)` on enums:
    on_enum: bool,
}

#[derive(Debug, Default)]
/// Represents the `derivative(Clone(…))` attributes on a field.
pub struct FieldClone {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `clone_with` attribute if present and the path to the cloning function.
    clone_with: Option<syn::Path>,
}

#[derive(Debug, Default)]
/// Represents the `derivative(Debug(…))` attributes on a field.
pub struct FieldDebug {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `format_with` attribute if present and the path to the formatting function.
    format_with: Option<syn::Path>,
    /// Whether the field is to be ignored from output.
    ignore: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Default(…))` attributes on a field.
pub struct FieldDefault {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The default value for the field if present.
    pub value: Option<proc_macro2::TokenStream>,
}

#[derive(Debug, Default)]
/// Represents the `derivative(Hash(…))` attributes on a field.
pub struct FieldHash {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `hash_with` attribute if present and the path to the hashing function.
    hash_with: Option<syn::Path>,
    /// Whether the field is to be ignored when hashing.
    ignore: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(PartialEq(…))` attributes on a field.
pub struct FieldPartialEq {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `compare_with` attribute if present and the path to the comparison function.
    compare_with: Option<syn::Path>,
    /// Whether the field is to be ignored when comparing.
    ignore: bool,
}

macro_rules! for_all_attr {
    (for ($name:ident, $value:ident) in $attrs:expr; $($body:tt)*) => {
        for meta_items in $attrs.iter().filter_map(|attr| derivative_attribute(attr.parse_meta())) {
            for meta_item in meta_items.iter().map(read_items) {
                let MetaItem($name, $value) = try!(meta_item);
                match $name.to_string().as_ref() {
                    $($body)*
                    _ => return Err(format!("unknown trait `{}`", $name)),
                }
            }
        }
    };
}

macro_rules! match_attributes {
    (let Some($name:ident) = $unwrapped:expr; for $value:ident in $values:expr; $($body:tt)* ) => {
        let mut $name = $unwrapped.take().unwrap_or_default();

        match_attributes! {
            for $value in $values;
            $($body)*
        }

        $unwrapped = Some($name);
    };

    (for $value:ident in $values:expr; $($body:tt)* ) => {
        for (name, $value) in $values {
            match name {
                Some(ident) => {
                    match ident.to_string().as_ref() {
                        $($body)*
                        _ => return Err(format!("unknown attribute `{}`", ident)),
                    }
                }
                None => {
                    match $value.expect("Expected value to be passed").value().as_ref() {
                        $($body)*
                        _ => return Err("unknown attribute".to_string()),
                    }
                }
            }
        }
    };
}

impl Input {
    /// Parse the `derivative` attributes on a type.
    pub fn from_ast(attrs: &[syn::Attribute]) -> Result<Input, String> {
        let mut input = Input::default();

        for_all_attr! {
            for (name, values) in attrs;
            "Clone" => {
                match_attributes! {
                    let Some(clone) = input.clone;
                    for value in values;
                    "bound" => try!(parse_bound(&mut clone.bounds, value)),
                    "clone_from" => {
                        clone.clone_from = try!(parse_boolean_meta_item(value, true, "clone_from"));
                    }
                }
            }
            "Copy" => {
                match_attributes! {
                    let Some(copy) = input.copy;
                    for value in values;
                    "bound" => try!(parse_bound(&mut copy.bounds, value)),
                }
            }
            "Debug" => {
                match_attributes! {
                    let Some(debug) = input.debug;
                    for value in values;
                    "bound" => try!(parse_bound(&mut debug.bounds, value)),
                    "transparent" => {
                        debug.transparent = try!(parse_boolean_meta_item(value, true, "transparent"));
                    }
                }
            }
            "Default" => {
                match_attributes! {
                    let Some(default) = input.default;
                    for value in values;
                    "bound" => try!(parse_bound(&mut default.bounds, value)),
                    "new" => {
                        default.new = try!(parse_boolean_meta_item(value, true, "new"));
                    }
                }
            }
            "Eq" => {
                match_attributes! {
                    let Some(eq) = input.eq;
                    for value in values;
                    "bound" => try!(parse_bound(&mut eq.bounds, value)),
                }
            }
            "Hash" => {
                match_attributes! {
                    let Some(hash) = input.hash;
                    for value in values;
                    "bound" => try!(parse_bound(&mut hash.bounds, value)),
                }
            }
            "PartialEq" => {
                match_attributes! {
                    let Some(partial_eq) = input.partial_eq;
                    for value in values;
                    "bound" => try!(parse_bound(&mut partial_eq.bounds, value)),
                    "feature_allow_slow_enum" => {
                        partial_eq.on_enum = try!(parse_boolean_meta_item(value, true, "feature_allow_slow_enum"));
                    }
                }
            }
        }

        Ok(input)
    }

    pub fn clone_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.clone
            .as_ref()
            .and_then(|d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn clone_from(&self) -> bool {
        self.clone.as_ref().map_or(false, |d| d.clone_from)
    }

    pub fn copy_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.copy
            .as_ref()
            .and_then(|d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn debug_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.debug
            .as_ref()
            .and_then(|d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn debug_transparent(&self) -> bool {
        self.debug.as_ref().map_or(false, |d| d.transparent)
    }

    pub fn default_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.default
            .as_ref()
            .and_then(|d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.eq
            .as_ref()
            .and_then(|d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn hash_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.hash
            .as_ref()
            .and_then(|d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn partial_eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.partial_eq
            .as_ref()
            .and_then(|d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn partial_eq_on_enum(&self) -> bool {
        self.partial_eq.as_ref().map_or(false, |d| d.on_enum)
    }
}

impl Field {
    /// Parse the `derivative` attributes on a type.
    pub fn from_ast(field: &syn::Field) -> Result<Field, String> {
        let mut out = Field::default();

        for_all_attr! {
            for (name, values) in field.attrs;
            "Clone" => {
                match_attributes! {
                    for value in values;
                    "bound" => try!(parse_bound(&mut out.clone.bounds, value)),
                    "clone_with" => {
                        let path = try!(value.ok_or_else(|| "`clone_with` needs a value".to_string()));
                        out.clone.clone_with = Some(try!(parse_str_lit(&path)));
                    }
                }
            }
            "Debug" => {
                match_attributes! {
                    for value in values;
                    "bound" => try!(parse_bound(&mut out.debug.bounds, value)),
                    "format_with" => {
                        let path = try!(value.ok_or_else(|| "`format_with` needs a value".to_string()));
                        out.debug.format_with = Some(try!(parse_str_lit(&path)));
                    }
                    "ignore" => {
                        out.debug.ignore = try!(parse_boolean_meta_item(value, true, "ignore"));
                    }
                }
            }
            "Default" => {
                match_attributes! {
                    for value in values;
                    "bound" => try!(parse_bound(&mut out.default.bounds, value)),
                    "value" => {
                        let value = try!(value.ok_or_else(|| "`value` needs a value".to_string()));
                        out.default.value = Some(try!(parse_str_lit(&value)));
                    }
                }
            }
            "Eq" => {
                match_attributes! {
                    for value in values;
                    "bound" => try!(parse_bound(&mut out.eq_bound, value)),
                }
            }
            "Hash" => {
                match_attributes! {
                    for value in values;
                    "bound" => try!(parse_bound(&mut out.hash.bounds, value)),
                    "hash_with" => {
                        let path = try!(value.ok_or_else(|| "`hash_with` needs a value".to_string()));
                        out.hash.hash_with = Some(try!(parse_str_lit(&path)));
                    }
                    "ignore" => {
                        out.hash.ignore = try!(parse_boolean_meta_item(value, true, "ignore"));
                    }
                }
            }
            "PartialEq" => {
                match_attributes! {
                    for value in values;
                    "bound" => try!(parse_bound(&mut out.partial_eq.bounds, value)),
                    "compare_with" => {
                        let path = try!(value.ok_or_else(|| "`compare_with` needs a value".to_string()));
                        out.partial_eq.compare_with = Some(try!(parse_str_lit(&path)));
                    }
                    "ignore" => {
                        out.partial_eq.ignore = try!(parse_boolean_meta_item(value, true, "ignore"));
                    }
                }
            }
        }

        Ok(out)
    }

    pub fn clone_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.clone.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn clone_with(&self) -> Option<&syn::Path> {
        self.clone.clone_with.as_ref()
    }

    pub fn copy_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.copy_bound.as_ref().map(Vec::as_slice)
    }

    pub fn debug_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.debug.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn debug_format_with(&self) -> Option<&syn::Path> {
        self.debug.format_with.as_ref()
    }

    pub fn ignore_debug(&self) -> bool {
        self.debug.ignore
    }

    pub fn ignore_hash(&self) -> bool {
        self.hash.ignore
    }

    pub fn default_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.default.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn default_value(&self) -> Option<&proc_macro2::TokenStream> {
        self.default.value.as_ref()
    }

    pub fn eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.eq_bound.as_ref().map(Vec::as_slice)
    }

    pub fn hash_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.hash.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn hash_with(&self) -> Option<&syn::Path> {
        self.hash.hash_with.as_ref()
    }

    pub fn partial_eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.partial_eq.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn partial_eq_compare_with(&self) -> Option<&syn::Path> {
        self.partial_eq.compare_with.as_ref()
    }

    pub fn ignore_partial_eq(&self) -> bool {
        self.partial_eq.ignore
    }
}

/// Represent an attribute.
///
/// We only have a limited set of possible attributes:
///
/// * `#[derivative(Debug)]` is represented as `(Debug, [])`;
/// * `#[derivative(Debug="foo")]` is represented as `(Debug, [(None, Some("foo"))])`;
/// * `#[derivative(Debug(foo="bar")]` is represented as `(Debug, [(Some(foo), Some("bar"))])`.
struct MetaItem<'a>(
    &'a syn::Ident,
    Vec<(Option<&'a syn::Ident>, Option<&'a syn::LitStr>)>,
);

/// Parse an arbitrary item for our limited `MetaItem` subset.
fn read_items(item: &syn::NestedMeta) -> Result<MetaItem, String> {
    let item = match *item {
        syn::NestedMeta::Meta(ref item) => item,
        syn::NestedMeta::Literal(..) => {
            return Err("Expected meta-item but found literal".to_string());
        }
    };
    match *item {
        syn::Meta::Word(ref name) => Ok(MetaItem(name, Vec::new())),
        syn::Meta::List(syn::MetaList {
            ident: ref name,
            nested: ref values,
            ..
        }) => {
            let values = try!(values
                .iter()
                .map(|value| {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                        ident: ref name,
                        lit: ref value,
                        ..
                    })) = *value
                    {
                        let value = try!(ensure_str_lit(&name.to_string(), value));

                        Ok((Some(name), Some(value)))
                    } else {
                        Err("Expected named value".to_string())
                    }
                })
                .collect());

            Ok(MetaItem(name, values))
        }
        syn::Meta::NameValue(syn::MetaNameValue {
            ident: ref name,
            lit: ref value,
            ..
        }) => {
            let value = try!(ensure_str_lit(&name.to_string(), value));

            Ok(MetaItem(name, vec![(None, Some(value))]))
        }
    }
}

/// Filter the `derivative` items from an attribute.
fn derivative_attribute(
    meta: syn::parse::Result<syn::Meta>,
) -> Option<syn::punctuated::Punctuated<syn::NestedMeta, syn::token::Comma>> {
    match meta {
        Ok(syn::Meta::List(syn::MetaList {
            ident: name,
            nested: mis,
            ..
        })) => {
            if name == "derivative" {
                Some(mis)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse an item value as a boolean. Accepted values are the string literal `"true"` and
/// `"false"`. The `default` parameter specifies what the value of the boolean is when only its
/// name is specified (eg. `Debug="ignore"` is equivalent to `Debug(ignore="true")`). The `name`
/// parameter is used for error reporting.
fn parse_boolean_meta_item(
    item: Option<&syn::LitStr>,
    default: bool,
    name: &str,
) -> Result<bool, String> {
    let item = item.map(|item| item.value());
    match item.as_ref().map(|item| item.as_ref()) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(val) => {
            if val == name {
                Ok(true)
            } else {
                Err(format!("Invalid value for `{}`: `{}`", name, val))
            }
        }
        None => Ok(default),
    }
}

/// Parse a `bound` item.
fn parse_bound(
    opt_bounds: &mut Option<Vec<syn::WherePredicate>>,
    value: Option<&syn::LitStr>,
) -> Result<(), String> {
    let bound = try!(value.ok_or_else(|| "`bound` needs a value".to_string()));

    let bound_value = bound.value();

    *opt_bounds = if !bound_value.is_empty() {
        let where_string = syn::LitStr::new(&format!("where {}", bound_value), bound.span());

        let bounds = parse_str_lit::<syn::WhereClause>(&where_string)
            .map(|wh| wh.predicates.into_iter().collect())
            .map_err(|_| "Could not parse `bound`".to_string());

        Some(try!(bounds))
    } else {
        Some(vec![])
    };

    Ok(())
}

fn parse_str_lit<T>(value: &syn::LitStr) -> Result<T, String>
where
    T: syn::parse::Parse,
{
    value.parse().map_err(|e| e.to_string())
}

fn ensure_str_lit<'a>(attr_name: &str, lit: &'a syn::Lit) -> Result<&'a syn::LitStr, String> {
    if let syn::Lit::Str(ref lit) = *lit {
        Ok(lit)
    } else {
        Err(format!(
            "expected derivative {} attribute to be a string: `{} = \"...\"`",
            attr_name, attr_name
        ))
    }
}
