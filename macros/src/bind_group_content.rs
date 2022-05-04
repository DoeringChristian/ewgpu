use quote::quote;

pub fn generate_bind_group_content(ast: syn::DeriveInput) -> proc_macro2::TokenStream{
    let ident = ast.ident;
    let data = ast.data;

    if let syn::Data::Struct(syn::DataStruct{fields, ..}) = data{

        let resources: Vec<proc_macro2::TokenStream> = fields.iter().enumerate()
            .map(|(i, field)|{
                let i = syn::Index::from(i);
                let ident = match &field.ident{
                    Some(ident) => quote!{#ident},
                    None => quote!{#i},
                };
                quote!{
                    self.#ident.resource()
                }
            }).collect();

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        let output = quote!{
            impl #impl_generics BindGroupContent for #ident #ty_generics #where_clause{
                fn resources<'br>(&'br self) -> Vec<wgpu::BindingResource<'br>>{
                    vec![#(#resources,)*]
                }
            }
        };

        return output;
    }

    panic!("Data type not supported");
}
