use quote::quote;
use super::utils::*;

pub fn generate_compute_data(ast: syn::DeriveInput) -> proc_macro2::TokenStream{

    let ident = ast.ident;
    let data = ast.data;

    if let syn::Data::Struct(syn::DataStruct{fields, ..}) = data{

        let bind_group_iter = fields.iter().filter(|x|{
            x.attrs.iter().find(|a|{
                a.path.is_ident("bind_group")
            }).is_some()
        });

        let bind_group_idents: Vec<proc_macro2::TokenStream> = bind_group_iter
            .clone()
            .map(|x|{
                let ident = &x.ident;
                quote!{
                    #ident
                }
            }).collect();

        let bind_group_paths: Vec<proc_macro2::TokenStream> = bind_group_iter
            .clone()
            .map(|x|{
                let path = reduce_to_path(&x.ty);
                quote!{
                    #path
                }
            }).collect();

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
        let output = quote!{
            impl #impl_generics PipelineData for #ident #ty_generics #where_clause{

            }
            impl #impl_generics ComputeData for #ident #ty_generics #where_clause{
                fn bind_groups<'d>(&'d self) -> Vec<&'d wgpu::BindGroup>{
                    vec![
                        #(self.#bind_group_idents.get_bind_group(),)*
                    ]
                }

                fn bind_group_layouts(device: &wgpu::Device) -> Vec<ewgpu::BindGroupLayoutWithDesc>{
                    vec![
                        #(
                            <#bind_group_paths>::create_bind_group_layout(device, None),
                        )*
                    ]
                }
            }
        };

        return output;

    }
    panic!("Data type not supported");
}
