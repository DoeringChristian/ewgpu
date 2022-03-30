use quote::quote;

pub fn generate_bind_group_content(ast: syn::DeriveInput) -> proc_macro2::TokenStream{
    let ident = ast.ident;
    let data = ast.data;

    if let syn::Data::Struct(syn::DataStruct{fields, ..}) = data{

        let mut entries = Vec::<proc_macro2::TokenStream>::new();
        let mut resources = Vec::<proc_macro2::TokenStream>::new();

        for field in fields{
            entries.push(generate_bind_group_entry(&field));
            resources.push(generate_bind_group_content_resource(&field));
        }

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        let output = quote!{
            impl #impl_generics BindGroupContent for #ident #ty_generics #where_clause{
                fn entries(visibility: wgpu::ShaderStages) -> Vec<BindGroupLayoutEntry>{
                    let mut ret = Vec::new();
                    #(#entries)*
                    ret
                }

                fn resources<'br>(&'br self) -> Vec<wgpu::BindingResource<'br>>{
                    let mut ret = Vec::new();
                    #(#resources)*
                    ret
                }
            }
        };

        return output.into();
    }

    panic!("Data type not supported");
}

pub fn generate_bind_group_entry(field: &syn::Field) -> proc_macro2::TokenStream{
    let ty = &field.ty;
    quote!{
        ret.append(&mut <#ty>::entries(visibility));
    }
}

pub fn generate_bind_group_content_resource(field: &syn::Field) -> proc_macro2::TokenStream{
    let ident = &field.ident;
    quote!{
        ret.append(&mut self.#ident.resources());
    }
}
