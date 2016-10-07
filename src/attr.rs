use syn;

#[derive(Debug, Default)]
pub struct Input {
    pub debug: Option<InputDebug>,
    pub default: bool,
}

#[derive(Debug, Default)]
pub struct Field {
    pub debug: Option<FieldDebug>,
    pub default: Option<FieldDefault>,
}

#[derive(Debug, Default)]
pub struct InputDebug {
    bounds: Option<Vec<syn::WherePredicate>>,
    pub transparent: bool,
}

#[derive(Debug, Default)]
pub struct FieldDebug {
    bounds: Option<Vec<syn::WherePredicate>>,
    format_with: Option<syn::Path>,
    ignore: bool,
}

#[derive(Debug, Default)]
pub struct FieldDefault {
    pub value: Option<syn::Expr>,
}

impl Input {
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
                                "bound" => {
                                    let mut bounds = debug.bounds.take().unwrap_or_default();
                                    try!(parse_bound(&mut bounds, value));
                                    debug.bounds = Some(bounds);
                                }
                                "transparent" => {
                                    debug.transparent = try!(parse_boolean_meta_item(&value, true, "transparent"));
                                }
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }

                        input.debug = Some(debug);
                    }
                    "Default" => {
                        input.default = true;
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
}

impl Field {
    pub fn from_ast(field: &syn::Field) -> Result<Field, String> {
        let mut out = Field::default();

        for meta_items in field.attrs.iter().filter_map(derivative_attribute) {
            for metaitem in meta_items.iter().map(read_items) {
                let MetaItem(name, values) = try!(metaitem);
                match name {
                    "Debug" => {
                        let mut debug = out.debug.take().unwrap_or_default();

                        for (name, value) in values {
                            match name {
                                "bound" => {
                                    let mut bounds = debug.bounds.take().unwrap_or_default();
                                    try!(parse_bound(&mut bounds, value));
                                    debug.bounds = Some(bounds);
                                }
                                "format_with" => {
                                    let path = try!(value.ok_or_else(|| "`format_with` needs a value".to_string()));
                                    debug.format_with = Some(try!(syn::parse_path(path)));
                                }
                                "ignore" => {
                                    debug.ignore = try!(parse_boolean_meta_item(&value, true, "ignore"));
                                }
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }

                        out.debug = Some(debug);
                    }
                    "Default" => {
                        let mut default = out.default.take().unwrap_or_default();

                        for (name, value) in values {
                            match name {
                                "value" => {
                                    let value = try!(value.ok_or_else(|| "`value` needs a value".to_string()));
                                    default.value = Some(try!(syn::parse_expr(value)));
                                }
                                _ => return Err(format!("unknown attribute `{}`", name)),
                            }
                        }

                        out.default = Some(default);
                    }
                    _ => return Err(format!("unknown trait `{}`", name)),
                }
            }
        }

        Ok(out)
    }

    pub fn debug_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.debug.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn debug_format_with(&self) -> Option<&syn::Path> {
        self.debug.as_ref().map_or(None, |d| d.format_with.as_ref())
    }

    pub fn ignore_debug(&self) -> bool {
        self.debug.as_ref().map_or(false, |debug| debug.ignore)
    }

    pub fn default_value(&self) -> Option<&syn::Expr> {
        self.default.as_ref().map_or(None, |d| d.value.as_ref())
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
                            Err(format!("Expected named value"))
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

fn parse_boolean_meta_item(item: &Option<&str>, default: bool, name: &str) -> Result<bool, String> {
    match *item {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(_) => Err(format!("Invalid value for `{}`", name)),
        None => Ok(default),
    }
}

fn parse_bound(bounds: &mut Vec<syn::WherePredicate>, value: Option<&str>) -> Result<(), String> {
    let bound = try!(value.ok_or_else(|| "`bound` needs a value".to_string()));
    let where_clause = syn::parse_where_clause(&format!("where {}", bound));
    let mut predicates = try!(where_clause).predicates;
    bounds.append(&mut predicates);
    Ok(())
}

fn str_or_err(lit: &syn::Lit) -> Result<&str, String> {
    if let syn::Lit::Str(ref value, _) = *lit {
        Ok(value.as_str())
    } else {
        Err(format!("Expected string"))
    }
}
