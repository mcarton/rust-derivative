use syn;

type Ctxt = ();

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
    pub fn from_ast(cx: &Ctxt, attrs: &[syn::Attribute]) -> Input {
        let mut input = Input::default();

        for meta_items in attrs.iter().filter_map(derivative_attribute) {
            for MetaItem(name, values) in meta_items.iter().map(read_items) {
                match name {
                    "Debug" => {
                        let mut debug = input.debug.take().unwrap_or_default();

                        for (name, value) in values {
                            match name {
                                "bound" => {
                                    let mut bounds = debug.bounds.take().unwrap_or_default();

                                    let clause = syn::parse_where_clause(&format!("where {}", value.unwrap()));
                                    bounds.append(&mut clause.unwrap().predicates);

                                    debug.bounds = Some(bounds);
                                }
                                "transparent" => {
                                    debug.transparent = match value {
                                        Some("true") | None => true,
                                        Some("false") => false,
                                        Some(_) => panic!(),
                                    };
                                }
                                _ => panic!(),
                            }
                        }

                        input.debug = Some(debug);
                    }
                    "Default" => {
                        input.default = true;
                    }
                    _ => panic!(),
                }
            }
        }

        input
    }

    pub fn debug_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.debug.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }
}

impl Field {
    pub fn from_ast(cx: &Ctxt, field: &syn::Field) -> Field {
        let mut out = Field::default();

        for meta_items in field.attrs.iter().filter_map(derivative_attribute) {
            for MetaItem(name, values) in meta_items.iter().map(read_items) {
                match name {
                    "Debug" => {
                        let mut debug = out.debug.take().unwrap_or_default();

                        for (name, value) in values {
                            match name {
                                "bound" => {
                                    let mut bounds = debug.bounds.take().unwrap_or_default();

                                    let clause = syn::parse_where_clause(&format!("where {}", value.unwrap()));
                                    bounds.append(&mut clause.unwrap().predicates);

                                    debug.bounds = Some(bounds);
                                }
                                "format_with" => {
                                    debug.format_with = Some(syn::parse_path(value.unwrap()).unwrap());
                                }
                                "ignore" => {
                                    debug.ignore = match value {
                                        Some("true") | None => true,
                                        Some("false") => false,
                                        Some(_) => panic!(),
                                    };
                                }
                                _ => panic!(),
                            }
                        }

                        out.debug = Some(debug);
                    }
                    "Default" => {
                        let mut default = out.default.take().unwrap_or_default();

                        for (name, value) in values {
                            match name {
                                "value" => {
                                    default.value = Some(syn::parse_expr(value.unwrap()).unwrap());
                                }
                                _ => panic!(),
                            }
                        }

                        out.default = Some(default);
                    }
                    _ => panic!(),
                }
            }
        }

        out
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

fn read_items(item: &syn::MetaItem) -> MetaItem {
    match *item {
        syn::MetaItem::Word(ref name) => MetaItem(name.as_ref(), Vec::new()),
        syn::MetaItem::List(ref name, ref values) => {
            let values = values
                .iter()
                .map(|value| {
                    match *value {
                        syn::MetaItem::Word(..) | syn::MetaItem::List(..) => panic!(),
                        syn::MetaItem::NameValue(ref name, ref value) => {
                            let value = if let syn::Lit::Str(ref value, _) = *value {
                                value.as_str()
                            } else {
                                panic!();
                            };

                            (name.as_ref(), Some(value))
                        }
                    }
                })
                .collect();

            MetaItem(name.as_ref(), values)
        }
        syn::MetaItem::NameValue(ref name, ref value) => {
            let value = if let syn::Lit::Str(ref value, _) = *value {
                value.as_str()
            } else {
                panic!();
            };

            MetaItem(name.as_ref(), vec![(value, None)])
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
