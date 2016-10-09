use syn;

/// Represent the `derivative` attributes on the input type (`struct`/`enum`).
#[derive(Debug, Default)]
pub struct Input {
    /// Whether `Debug` is present and its specific attributes.
    pub debug: Option<InputDebug>,
    /// Whether `Default` is present and its specitif attributes.
    pub default: Option<InputDefault>,
    /// Whether `Eq` is present and its specitif attributes.
    pub eq: Option<InputEq>,
}

#[derive(Debug, Default)]
/// Represent the `derivative` attributes on a field.
pub struct Field {
    /// The parameters for `Debug`.
    debug: FieldDebug,
    /// The parameters for `Default`.
    default: FieldDefault,
    /// The parameters for `Eq`.
    eq_bound: Option<Vec<syn::WherePredicate>>,
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
/// Represents the `derivarive(Debug(…))` attributes on a field.
pub struct FieldDebug {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `format_with` attribute if present and the path to the formatting function.
    format_with: Option<syn::Path>,
    /// Whether the field is to be ignore from output.
    ignore: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivarive(Default(…))` attributes on a field.
pub struct FieldDefault {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The default value for the field if present.
    pub value: Option<syn::Expr>,
}

#[derive(Debug, Default)]
/// Represent the `derivarive(Eq(…))` attributes on a field.
pub struct FieldEq {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
}

impl Input {
    /// Parse the `derivative` attributes on a type.
    pub fn from_ast(attrs: &[syn::Attribute]) -> Result<Input, String> {
        let mut input = Input::default();

        for meta_items in attrs.iter().filter_map(derivative_attribute) {
            for metaitem in meta_items.iter().map(read_items) {
                let MetaItem(name, values) = try!(metaitem);
                match name {
                    "Debug" => {
                        let mut debug = input.debug.take().unwrap_or_default();

                        for (name, value) in values {
                            match name {
                                "bound" => try!(parse_bound(&mut debug.bounds, value)),
                                "transparent" => {
                                    debug.transparent = try!(parse_boolean_meta_item(&value, true, "transparent"));
                                }
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }

                        input.debug = Some(debug);
                    }
                    "Default" => {
                        let mut default = input.default.take().unwrap_or_default();

                        for (name, value) in values {
                            match name {
                                "bound" => try!(parse_bound(&mut default.bounds, value)),
                                "new" => {
                                    default.new = try!(parse_boolean_meta_item(&value, true, "new"));
                                }
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }

                        input.default = Some(default);
                    }
                    "Eq" => {
                        let mut eq = input.eq.take().unwrap_or_default();

                        for (name, value) in values {
                            match name {
                                "bound" => try!(parse_bound(&mut eq.bounds, value)),
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }

                        input.eq = Some(eq);
                    }
                    _ => return Err(format!("unknown trait `{}`", name)),
                }
            }
        }

        Ok(input)
    }

    pub fn debug_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.debug.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn debug_transparent(&self) -> bool {
        self.debug.as_ref().map_or(false, |d| d.transparent)
    }

    pub fn default_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.default.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.eq.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }
}

impl Field {
    /// Parse the `derivative` attributes on a type.
    pub fn from_ast(field: &syn::Field) -> Result<Field, String> {
        let mut out = Field::default();

        for meta_items in field.attrs.iter().filter_map(derivative_attribute) {
            for metaitem in meta_items.iter().map(read_items) {
                let MetaItem(name, values) = try!(metaitem);
                match name {
                    "Debug" => {
                        for (name, value) in values {
                            match name {
                                "bound" => try!(parse_bound(&mut out.debug.bounds, value)),
                                "format_with" => {
                                    let path = try!(value.ok_or_else(|| "`format_with` needs a value".to_string()));
                                    out.debug.format_with = Some(try!(syn::parse_path(path)));
                                }
                                "ignore" => {
                                    out.debug.ignore = try!(parse_boolean_meta_item(&value, true, "ignore"));
                                }
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }
                    }
                    "Default" => {
                        for (name, value) in values {
                            match name {
                                "bound" => try!(parse_bound(&mut out.default.bounds, value)),
                                "value" => {
                                    let value = try!(value.ok_or_else(|| "`value` needs a value".to_string()));
                                    out.default.value = Some(try!(syn::parse_expr(value)));
                                }
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }
                    }
                    "Eq" => {
                        for (name, value) in values {
                            match name {
                                "bound" => try!(parse_bound(&mut out.eq_bound, value)),
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }
                    }
                    _ => return Err(format!("unknown trait `{}`", name)),
                }
            }
        }

        Ok(out)
    }

    pub fn debug_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.debug.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.eq_bound.as_ref().map(Vec::as_slice)
    }

    pub fn debug_format_with(&self) -> Option<&syn::Path> {
        self.debug.format_with.as_ref()
    }

    pub fn ignore_debug(&self) -> bool {
        self.debug.ignore
    }

    pub fn default_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.default.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn default_value(&self) -> Option<&syn::Expr> {
        self.default.value.as_ref()
    }
}

/// Represent an attribute.
///
/// We only have a limited set of possible attributes:
///
/// * `#[derivative(Debug)]` is represented as `("Debug", [])`;
/// * `#[derivative(Debug="foo")]` is represented as `("Debug", [("foo", None)])`;
/// * `#[derivative(Debug(foo="bar")]` is represented as `("Debug", [("foo", Some("bar"))])`.
struct MetaItem<'a>(&'a str, Vec<(&'a str, Option<&'a str>)>);

/// Parse an arbitrary item for our limited `MetaItem` subset.
fn read_items(item: &syn::MetaItem) -> Result<MetaItem, String> {
    match *item {
        syn::MetaItem::Word(ref name) => Ok(MetaItem(name.as_ref(), Vec::new())),
        syn::MetaItem::List(ref name, ref values) => {
            let values = try!(
                values
                .iter()
                .map(|value| {
                    match *value {
                        syn::MetaItem::Word(..) | syn::MetaItem::List(..) => {
                            Err("Expected named value".to_string())
                        }
                        syn::MetaItem::NameValue(ref name, ref value) => {
                            let value = try!(str_or_err(value));

                            Ok((name.as_ref(), Some(value)))
                        }
                    }
                })
                .collect()
            );

            Ok(MetaItem(name.as_ref(), values))
        }
        syn::MetaItem::NameValue(ref name, ref value) => {
            let value = try!(str_or_err(value));

            Ok(MetaItem(name.as_ref(), vec![(value, None)]))
        }
    }
}

/// Filter the `derivative` items from an attribute.
fn derivative_attribute(attr: &syn::Attribute) -> Option<&[syn::MetaItem]> {
    match attr.value {
        syn::MetaItem::List(ref name, ref mis) if name == "derivative" => {
            Some(mis)
        }
        syn::MetaItem::Word(..) |
        syn::MetaItem::NameValue(..) |
        syn::MetaItem::List(..) => None,
    }
}

/// Parse an item value as a boolean. Accepted values are the string literal `"true"` and
/// `"false"`. The `default` parameter specifies what the value of the boolean is when only its
/// name is specified (eg. `Debug="ignore"` is equivalent to `Debug(ignore="true")`). The `name`
/// parameter is used for error reporting.
fn parse_boolean_meta_item(item: &Option<&str>, default: bool, name: &str) -> Result<bool, String> {
    match *item {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(_) => Err(format!("Invalid value for `{}`", name)),
        None => Ok(default),
    }
}

/// Parse a `bound` item.
fn parse_bound(
    opt_bounds: &mut Option<Vec<syn::WherePredicate>>,
    value: Option<&str>
) -> Result<(), String> {
    let mut bounds = opt_bounds.take().unwrap_or_default();
    let bound = try!(value.ok_or_else(|| "`bound` needs a value".to_string()));

    if !bound.is_empty() {
        let where_clause = syn::parse_where_clause(&format!("where {}", bound));
        let mut predicates = try!(where_clause).predicates;
        bounds.append(&mut predicates);
    }

    *opt_bounds = Some(bounds);

    Ok(())
}

/// Get the string out of a string literal or report an error for other literals.
fn str_or_err(lit: &syn::Lit) -> Result<&str, String> {
    if let syn::Lit::Str(ref value, _) = *lit {
        Ok(value.as_str())
    } else {
        Err("Expected string".to_string())
    }
}
