use quote::quote;

pub fn generate_deref(ast: syn::DeriveInput) -> proc_macro2::TokenStream{
    let ident = ast.ident;
    let data = ast.data;

    match data{
        syn::Data::Struct(syn::DataStruct{fields, ..}) => {
            let target = fields.iter().find(|f|{
                f.attrs.iter().find(|a|{
                    a.path.is_ident("target")
                }).is_some()
            }).unwrap_or_else(||{
                if fields.iter().count() == 1{
                    fields.iter().next().unwrap()
                }
                else{
                    panic!("If there are more than one field add a #[target] attribute.")
                }
            });

            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

            let target_ty = &target.ty;
            let target_ident = &target.ident;

            quote!{
                impl #impl_generics std::ops::Deref for #ident #ty_generics #where_clause{
                    type Target = #target_ty;
                    #[inline]
                    fn deref(&self) -> &Self::Target{
                        &self.#target_ident
                    }
                }
            }
        },
        _ => panic!("Only supported for structs"),
    }
}

pub fn generate_derefmut(ast: syn::DeriveInput) -> proc_macro2::TokenStream{
    let ident = ast.ident;
    let data = ast.data;

    match data{
        syn::Data::Struct(syn::DataStruct{fields, ..}) => {
            let target = fields.iter().find(|f|{
                f.attrs.iter().find(|a|{
                    a.path.is_ident("target")
                }).is_some()
            }).unwrap_or_else(||{
                if fields.iter().count() == 1{
                    fields.iter().next().unwrap()
                }
                else{
                    panic!("If there are more than one field add a #[target] attribute.")
                }
            });

            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

            let target_ty = &target.ty;
            let target_ident = &target.ident;

            quote!{
                impl #impl_generics std::ops::Deref for #ident #ty_generics #where_clause{
                    type Target = #target_ty;
                    #[inline]
                    fn deref(&self) -> &Self::Target{
                        &self.#target_ident
                    }
                }

                impl #impl_generics std::ops::DerefMut for #ident #ty_generics #where_clause{
                    #[inline]
                    fn deref_mut(&mut self) -> &mut Self::Target{
                        &mut self.#target_ident
                    }
                }
            }
        },
        _ => panic!("Only supported for structs"),
    }
}
