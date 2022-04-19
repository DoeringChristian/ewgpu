
pub fn reduce_to_path(ty: &syn::Type) -> &syn::Path{
    match ty{
        syn::Type::Path(type_path) => &type_path.path,
        syn::Type::Reference(reference) => reduce_to_path(reference.elem.as_ref()),
        _ => panic!("Unsupported type")
    }
}

