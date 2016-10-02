use syn;

pub fn collect_derive_attrs(mut input: syn::MacroInput) -> (syn::MacroInput, Vec<syn::MetaItem>) {
    let mut attrs = Vec::new();

    input.attrs.retain(|attr| {
        if let Some(mis) = derivative_attribute(&attr) {
            attrs.extend_from_slice(mis);
            false
        } else {
            true
        }
    });

    (input, attrs)
}

pub fn derivative_attribute(attr: &syn::Attribute) -> Option<&[syn::MetaItem]> {
    match attr.value {
        syn::MetaItem::List(ref name, ref mis) if name == "derivative" => {
            Some(mis)
        }
        syn::MetaItem::Word(..) |
        syn::MetaItem::NameValue(..) |
        syn::MetaItem::List(..) => None,
    }
}

pub fn ignored_traits(attrs: &[syn::Attribute]) -> Vec<&str> {
    attrs.iter().filter_map(derivative_attribute).flat_map(|a| a).map(|attr| {
        match *attr {
            syn::MetaItem::List(ref name, ref mis) if name == "ignore_for" => {
                mis.iter().map(|mi| {
                    if let syn::MetaItem::Word(ref name) = *mi {
                        name.as_ref()
                    } else {
                        panic!()
                    }
                }).collect()
            }
            _ => Vec::new(),
        }
    }).flat_map(|s| s).collect()
}
