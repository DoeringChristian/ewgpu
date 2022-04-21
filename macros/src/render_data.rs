use quote::quote;
use super::utils::*;

pub fn generate_render_data(ast: syn::DeriveInput) -> proc_macro2::TokenStream{

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

        let bind_group_visibilities: Vec<proc_macro2::TokenStream> = bind_group_iter
            .clone()
            .map(|x|{
                let attr = x.attrs.iter().find(|x|{
                    x.path.is_ident("bind_group")
                }).unwrap();

                let vis = attr.parse_meta().map(|m|{
                    match m{
                        syn::Meta::NameValue(n) => {
                            if let syn::Lit::Str(i) = n.lit{
                                let content: syn::Ident = i.parse().unwrap();
                                quote!{
                                    #content
                                }
                            }
                            else{
                                panic!("Invalid literal provided");
                            }
                        },
                        _ => quote!{all()},
                    }
                }).unwrap_or_else(|_|{
                    quote!{
                        all()
                    }
                });
                quote!{
                    wgpu::ShaderStages::#vis
                }
            }).collect();

        let vertex_slices_iter = fields.iter().filter(|x|{
            x.attrs.iter().find(|a|{
                a.path.is_ident("vertex")
            }).is_some()
        });

        let vertex_slices_idents: Vec<proc_macro2::TokenStream> = vertex_slices_iter
            .clone()
            .map(|x|{
                let ident = &x.ident;
                quote!{
                    #ident
                }
            }).collect();

        let vertex_slice_paths: Vec<proc_macro2::TokenStream> = vertex_slices_iter
            .clone()
            .map(|x|{
                let path = reduce_to_path(&x.ty);
                quote!{
                    #path
                }
            }).collect();

        let index_slice_iter = fields.iter().filter(|x|{
            x.attrs.iter().find(|a|{
                a.path.is_ident("index")
            }).is_some()
        });

        let index_slice_ident: proc_macro2::TokenStream = index_slice_iter
            .clone()
            .map(|x|{
                let ident = &x.ident;
                quote!{
                    #ident
                }
            }).next().unwrap();

        let index_slice_path: proc_macro2::TokenStream = index_slice_iter
            .clone()
            .map(|x|{
                let path = reduce_to_path(&x.ty);
                quote!{
                    #path
                }
            }).next().unwrap();


        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
        let output = quote!{
            impl #impl_generics PipelineData for #ident #ty_generics #where_clause{

            }
            impl #impl_generics RenderData for #ident #ty_generics #where_clause{
                fn bind_groups<'d>(&'d self) -> Vec<&'d wgpu::BindGroup>{
                    vec![
                        #(self.#bind_group_idents.get_bind_group(),)*
                    ]
                }

                fn bind_group_layouts(device: &wgpu::Device) -> Vec<ewgpu::BindGroupLayoutWithDesc>{
                    vec![
                        #(
                            <#bind_group_paths>::create_bind_group_layout(device, None, #bind_group_visibilities),
                        )*
                    ]
                }

                fn vert_layout() -> Vec<wgpu::VertexBufferLayout<'static>>{
                    vec![
                        #(<#vertex_slice_paths>::buffer_slice_vert_layout(),)*
                    ]
                }

                fn vert_buffer_slices<'d>(&'d self) -> Vec<wgpu::BufferSlice<'d>>{
                    vec![
                        #(self.#vertex_slices_idents.slice(),)*
                    ]
                }

                fn idx_buffer_slice<'d>(&'d self) -> (wgpu::IndexFormat, wgpu::BufferSlice<'d>) {
                    (<#index_slice_path>::index_buffer_format(), self.#index_slice_ident.slice())
                }


            }
        };

        return output;

    }
    panic!("Data type not supported");
}
