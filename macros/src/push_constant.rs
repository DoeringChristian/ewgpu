use proc_macro::{self, TokenStream};
use quote::quote;

pub fn generate_push_constant(ast: syn::DeriveInput) -> proc_macro2::TokenStream{

    let ident = ast.ident;
    let data = ast.data;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let output = quote!{
        impl #impl_generics PushConstant for #ident #ty_generics #where_clause{
            fn push_const_ranges() -> Vec<wgpu::PushConstantRange>{
                todo!()
            }
            fn push_to<'rp>(&self, render_pass_pipeline: &mut RenderPassPipeline<'rp, '_>){
                todo!()
            }
        }
    };

    output
}

fn generate_range(field: &syn::Field) -> proc_macro2::TokenStream{

    let ident = &field.ident;
    let ty = &field.ty;

    quote!{
        wgpu::PushConstantRange{
            stages: wgpu::ShaderStage::all(),
            range: Range<u32>{
            }
        }
    }
}
