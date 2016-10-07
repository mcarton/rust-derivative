use ast;
use attr;
use bound;
use syn;

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

pub fn remove_derivative_attrs(input: &mut syn::MacroInput) {
    fn remove_from_vec(attrs: &mut Vec<syn::Attribute>) {
        attrs.retain(|attr| derivative_attribute(attr).is_none());
     }

    fn remove_from_variant_data(vd: &mut syn::VariantData) {
        match *vd {
            syn::VariantData::Struct(ref mut fields) | syn::VariantData::Tuple(ref mut fields) => {
                for field in fields {
                    remove_from_vec(&mut field.attrs);
                }
            }
            syn::VariantData::Unit => (),
        }
    }

    remove_from_vec(&mut input.attrs);

    match input.body {
        syn::Body::Enum(ref mut variants) => {
            for variant in variants {
                remove_from_vec(&mut variant.attrs);
                remove_from_variant_data(&mut variant.data);
            }
        }
        syn::Body::Struct(ref mut vd) => {
            remove_from_variant_data(vd);
        }
    }
 }

/// Make generic with all the generics in the input, plus a bound `T: <trait_path>` for each
/// generic field type that will be shown.
pub fn build_impl_generics<F, G, H>(
    item: &ast::Input,
    trait_path: &syn::Path,
    needs_debug_bound: F,
    field_bound: G,
    input_bound: H,
) -> syn::Generics
    where F: Fn(&attr::Field) -> bool,
          G: Fn(&attr::Field) -> Option<&[syn::WherePredicate]>,
          H: Fn(&attr::Input) -> Option<&[syn::WherePredicate]>,
{
    let generics = bound::without_defaults(item.generics);
    let generics = bound::with_where_predicates_from_fields(
        item, &generics, field_bound
    );

    match input_bound(&item.attrs) {
        Some(predicates) => {
            bound::with_where_predicates(&generics, predicates)
        }
        None => {
            bound::with_bound(item, &generics, needs_debug_bound, trait_path)
        }
    }
}
