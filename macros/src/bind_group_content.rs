use quote::quote;

pub fn generate_bind_group_content(ast: syn::DeriveInput) -> proc_macro2::TokenStream{
    let ident = ast.ident;
    let data = ast.data;

    if let syn::Data::Struct(syn::DataStruct{fields, ..}) = data{

        let resources: Vec<proc_macro2::TokenStream> = fields.iter()
            .map(|field|{
                let ident = &field.ident;
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

fn parse_meta_own(attr: &syn::Attribute) -> syn::Meta {
    fn clone_ident_segment(segment: &syn::PathSegment) -> syn::PathSegment {
        syn::PathSegment {
            ident: segment.ident.clone(),
            arguments: syn::PathArguments::None,
        }
    }

    let path = syn::Path {
        leading_colon: attr
            .path
            .leading_colon
            .as_ref()
            .map(|colon| syn::Token![::](colon.spans)),
            segments: attr
                .path
                .segments
                .pairs()
                .map(|pair| match pair {
                    syn::punctuated::Pair::Punctuated(seg, punct) => {
                        syn::punctuated::Pair::Punctuated(clone_ident_segment(seg), syn::Token![::](punct.spans))
                    }
                    syn::punctuated::Pair::End(seg) => syn::punctuated::Pair::End(clone_ident_segment(seg)),
                })
        .collect(),
    };
    todo!()
}

pub fn generate_bind_group_entry(field: &syn::Field) -> proc_macro2::TokenStream{
    let visibility = field.attrs.iter().find(|a|{
        a.path.is_ident("visibility")
    });
    let ty = &field.ty;

    match visibility{
        Some(visibility) => {
            let visibility: proc_macro2::TokenStream = visibility.parse_meta().map(|m|{
                match m{
                    syn::Meta::NameValue(n) => {
                        if let syn::Lit::Str(i) = n.lit{
                            let content: syn::Expr = i.parse().expect("Error not an Ident");
                            quote!{
                                #content
                            }
                        }
                        else{
                            panic!("Invalid literal provided");
                        }
                    },
                    _ => quote!{wgpu::ShaderStages::all()},
                }
            }).unwrap_or_else(|_|{
                quote!{
                    wgpu::ShaderStages::all()
                }
            });
            quote!{
                ret.append(&mut <#ty>::entries(Some(#visibility)));
            }
        },
        None => {
            quote!{
                ret.append(&mut <#ty>::entries(None));
            }
        }
    }
}

pub fn generate_bind_group_content_resource(field: &syn::Field) -> proc_macro2::TokenStream{
    let ident = &field.ident;
    quote!{
        ret.append(&mut self.#ident.resource());
    }
}
