use proc_macro::{self, TokenStream};

mod vertex;

use vertex::*;

///
/// A Macro for deriving A instance vector from a struct:
///
/// ```
/// #[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Instance)]
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
