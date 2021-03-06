use quote::quote;

pub fn generate_instance(ast: syn::DeriveInput) -> proc_macro2::TokenStream{
    let ident = ast.ident;
    let data = ast.data;

    let mut attributes = Vec::<proc_macro2::TokenStream>::new();

    if let syn::Data::Struct(syn::DataStruct{fields, ..}) = data{
        for field in fields{
            attributes.push(generate_vertex_attributes(&field));
        }

        let len = attributes.len();

        let(impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        let output = quote!{
            impl #impl_generics VertLayout for #ident #ty_generics #where_clause{
                fn buffer_layout() -> wgpu::VertexBufferLayout<'static>{
                    const ATTRIBS: [wgpu::VertexAttribute; #len] = wgpu::vertex_attr_array!(
                        #(#attributes)*
                    );
                    wgpu::VertexBufferLayout{
                        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &ATTRIBS,
                    }
                }
            }
        };
        return output;
    }
    panic!("Data type not supported");
}

pub fn generate_vert(ast: syn::DeriveInput) -> proc_macro2::TokenStream{
    let ident = ast.ident;
    let data = ast.data;

    let mut attributes = Vec::<proc_macro2::TokenStream>::new();

    if let syn::Data::Struct(syn::DataStruct{fields, ..}) = data{
        for field in fields{
            attributes.push(generate_vertex_attributes(&field));
        }

        let len = attributes.len();

        let(impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        let output = quote!{
            impl #impl_generics VertLayout for #ident #ty_generics #where_clause{
                fn buffer_layout() -> wgpu::VertexBufferLayout<'static>{
                    const ATTRIBS: [wgpu::VertexAttribute; #len] = wgpu::vertex_attr_array!(
                        #(#attributes)*
                    );
                    wgpu::VertexBufferLayout{
                        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &ATTRIBS,
                    }
                }
            }
        };
        return output;
    }
    panic!("Data type not supported");
}

pub fn generate_vertex_attributes(field: &syn::Field) -> proc_macro2::TokenStream{
    let field_name = match field.ident{
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };

    let location_attr = field.attrs.iter().find(|x| x.path.is_ident("location")).unwrap_or_else(|| panic!("Field {} has no location attribute", field_name));

    let meta = location_attr.parse_meta().unwrap_or_else(|_| panic!("Field {} does not have a attribute that conforms to structured format.", field_name));

    let location_value: u32 = match meta{
        syn::Meta::NameValue(meta_named_value) => {
            if let syn::Lit::Int(i) = meta_named_value.lit{
                i.base10_parse().unwrap_or_else(|_| panic!("Field {} location attribute value must be a base10 integer", field_name))
            }else{
                panic!("Field {} location attribute value must be an integer", field_name);
            }
        }
        _ => panic!("Field {} location attribute value must be a string literal", field_name),
    };

    let norm_attr = field.attrs.iter().any(|x| x.path.is_ident("norm"));

    let format = format(field, norm_attr);

    let output = quote!{
        #location_value => #format,
    };

    output

}

pub fn format(field: &syn::Field, norm: bool) -> proc_macro2::TokenStream{
    let field_name = match field.ident{
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };

    match &field.ty{
        syn::Type::Path(type_path) => {
            if type_path.path.is_ident("u32"){
                return quote!{Uint32};
            }
            if type_path.path.is_ident("i32"){
                return quote!{Sint32};
            }
            if type_path.path.is_ident("f32"){
                return quote!{Float32};
            }
            if type_path.path.is_ident("u64"){
                return quote!{Float64};
            }
        }
        syn::Type::Array(type_array) => {
            if let syn::Expr::Lit(syn::ExprLit{lit: syn::Lit::Int(lit_int), ..}) = &type_array.len{

                let array_len: u32 = lit_int.base10_parse().unwrap_or_else(|_| panic!("Field {} array's length has to be known at compiletime.", field_name));

                match &(*type_array.elem){
                    syn::Type::Path(type_path) => {
                        if type_path.path.is_ident("u8"){
                            if norm{
                                match array_len{
                                    2 => return quote!{Unorm8x2},
                                    4 => return quote!{Unorm8x4},
                                    _ => panic!("Field {} array of this length is not supported", field_name),
                                }
                            }
                            else{
                                match array_len{
                                    2 => return quote!{Uint8x2},
                                    4 => return quote!{Uint8x4},
                                    _ => panic!("Field {} array of this length is not supported", field_name),
                                }
                            }
                        }
                        if type_path.path.is_ident("i8"){
                            if norm{
                                match array_len{
                                    2 => return quote!{Snorm8x2},
                                    4 => return quote!{Snorm8x4},
                                    _ => panic!("Field {} array of this length is not supported", field_name),
                                }
                            }
                            else{
                                match array_len{
                                    2 => return quote!{Sint8x2},
                                    4 => return quote!{Sint8x4},
                                    _ => panic!("Field {} array of this length is not supported", field_name),
                                }
                            }
                        }
                        if type_path.path.is_ident("u16"){
                            if norm{
                                match array_len{
                                    2 => return quote!{Unorm16x2},
                                    4 => return quote!{Unorm16x4},
                                    _ => panic!("Field {} array of this length is not supported", field_name),
                                }
                            }
                            else{
                                match array_len{
                                    2 => return quote!{Uint16x2},
                                    4 => return quote!{Uint16x4},
                                    _ => panic!("Field {} array of this length is not supported", field_name),
                                }
                            }
                        }
                        if type_path.path.is_ident("i16"){
                            if norm{
                                match array_len{
                                    2 => return quote!{Snorm16x2},
                                    4 => return quote!{Snorm16x4},
                                    _ => panic!("Field {} array of this length is not supported", field_name),
                                }
                            }
                            else{
                                match array_len{
                                    2 => return quote!{Sint16x2},
                                    4 => return quote!{Sint16x4},
                                    _ => panic!("Field {} array of this length is not supported", field_name),
                                }
                            }
                        }
                        if type_path.path.is_ident("u32"){
                            match array_len{
                                1 => return quote!{Uint32},
                                2 => return quote!{Uint32x2},
                                3 => return quote!{Uint32x3},
                                4 => return quote!{Uint32x4},
                                _ => panic!("Field {} array of this length is not supported", field_name),
                            }
                        }
                        if type_path.path.is_ident("i32"){
                            match array_len{
                                1 => return quote!{Sint32},
                                2 => return quote!{Sint32x2},
                                3 => return quote!{Sint32x3},
                                4 => return quote!{Sint32x4},
                                _ => panic!("Field {} array of this length is not supported", field_name),
                            }
                        }
                        if type_path.path.is_ident("f32"){
                            match array_len{
                                1 => return quote!{Float32},
                                2 => return quote!{Float32x2},
                                3 => return quote!{Float32x3},
                                4 => return quote!{Float32x4},
                                _ => panic!("Field {} array of this length is not supported", field_name),
                            }
                        }
                        if type_path.path.is_ident("f64"){
                            match array_len{
                                1 => return quote!{Float64},
                                2 => return quote!{Float64x2},
                                3 => return quote!{Float64x3},
                                4 => return quote!{Float64x4},
                                _ => panic!("Field {} array of this length is not supported", field_name),
                            }
                        }
                    }
                    _ => panic!("Field {} with array of this type is not supported", field_name),
                }
            }
        },
        _ => panic!("Field {} type is not a valid Format.", field_name),
    }
    todo!()
}


#[cfg(test)]
mod tests {

}
