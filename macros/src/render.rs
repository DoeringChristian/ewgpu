use proc_macro::{self, TokenStream};
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

pub fn generate_render(ast: syn::DeriveInput) -> proc_macro2::TokenStream{
    let ident = ast.ident;
    let data = ast.data;

    let(impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let output = quote!{
        impl #impl_generics Drawable for #ident #ty_generics #where_clause{
            fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>){

            }

            fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>>{
                Self::create_vert_buffer_layouts()
            }
            fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>>{

            }
        }
    };

    output
}
