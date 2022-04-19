use proc_macro::{self, TokenStream};

mod vertex;
mod bind_group_content;
mod pipeline_layout;
mod render_data;
mod compute_data;
mod utils;

use vertex::*;
use bind_group_content::*;
use pipeline_layout::*;
use render_data::*;
use compute_data::*;

///
/// An attribute macro for deriving Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Vert
///
/// ```
/// #[make_vert]
/// struct Vert{
///     #[location = 0]
///     pub uint8: u8,
///     #[location = 1]
///     pub uint8x2: [u8; 2],
///     #[location = 2]
///     pub uint8x4: [u8; 4],
///     #[location = 3]
///     pub unorm8x2: [u8; 2],
///     #[location = 4]
///     pub unorm8x4: [u8; 2],
/// }
///
/// let layout = Vert::buffer_layout();
/// ```
///
#[proc_macro_attribute]
pub fn make_vert(_metadata: TokenStream, input: TokenStream) -> TokenStream{
    let input: proc_macro2::TokenStream = input.into();

    quote::quote!{
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Vert)]
        #input
    }.into()
}

///
/// An attribute macro for deriving Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Instance
///
/// ```
/// #[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Inst)]
/// struct Inst{
///     #[location = 0]
///     pub uint8: u8,
///     #[location = 1]
///     pub uint8x2: [u8; 2],
///     #[location = 2]
///     pub uint8x4: [u8; 4],
///     #[location = 3]
///     pub unorm8x2: [u8; 2],
///     #[location = 4]
///     pub unorm8x4: [u8; 2],
/// }
///
/// let layout = Inst::buffer_layout();
/// ```
///
#[proc_macro_attribute]
pub fn make_inst(_metadata: TokenStream, input: TokenStream) -> TokenStream{
    let input: proc_macro2::TokenStream = input.into();

    quote::quote!{
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Inst)]
        #input
    }.into()
}

///
/// A Macro for deriving A instance vector from a struct:
///
/// ```
/// #[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Inst)]
/// struct Inst{
///     #[location = 0]
///     pub uint8: u8,
///     #[location = 1]
///     pub uint8x2: [u8; 2],
///     #[location = 2]
///     pub uint8x4: [u8; 4],
///     #[location = 3]
///     pub unorm8x2: [u8; 2],
///     #[location = 4]
///     pub unorm8x4: [u8; 2],
/// }
///
/// let layout = Inst::buffer_layout();
/// ```
///
#[proc_macro_derive(Inst, attributes(location, norm))]
pub fn derive_instance(tokens: TokenStream) -> TokenStream{
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();

    generate_instance(ast).into()
}

#[proc_macro_derive(Vert, attributes(location, norm))]
pub fn derive_vert(tokens: TokenStream) -> TokenStream{
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();

    generate_vert(ast).into()
}

///
/// A macro to derive the BindGroupContent trait from a struct whos fields implement
/// BindGroupContent.
///
/// ```
/// #[derive(BindGroupContent)]
/// struct TestBindGroupContent{
///     indices: Buffer<u32>,
///     other_indices: Buffer<u64>,
/// }
/// ```
///
#[proc_macro_derive(BindGroupContent)]
pub fn derive_bind_group_content(tokens: TokenStream) -> TokenStream{
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();

    generate_bind_group_content(ast).into()
}

#[proc_macro]
pub fn pipeline_layout(tokens: TokenStream) -> TokenStream{
    generate_pipeline_layout(tokens)
}

#[proc_macro_derive(RenderData, attributes(bind_group, vertex, index))]
pub fn derive_render_data(tokens: TokenStream) -> TokenStream{
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();

    generate_render_data(ast).into()
}

#[proc_macro_derive(ComputeData, attributes(bind_group))]
pub fn derive_compute_data(tokens: TokenStream) -> TokenStream{
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();

    generate_compute_data(ast).into()
}


