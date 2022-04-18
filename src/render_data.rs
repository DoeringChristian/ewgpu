use std::ops::Deref;

use crate::*;

pub trait RenderData{
    fn bind_groups<'d>(&'d self) -> Vec<&'d wgpu::BindGroup>;
    fn bind_group_layouts(device: &wgpu::Device) -> Vec<BindGroupLayoutWithDesc>;
    fn vert_buffer_slices<'d>(&'d self) -> Vec<wgpu::BufferSlice<'d>>{
        vec![]
    }
    fn idx_buffer_slice<'d>(&'d self) -> (wgpu::IndexFormat, wgpu::BufferSlice<'d>);
}

/*
macro_rules! render_data_for_tuple{
    ($($name:ident)+) => {
        #[allow(non_snake_case)]
        impl<'rd, $($name: BindGroupContent),+> RenderData for ($(&'rd BindGroup<$name>, )+){
            fn bind_groups<'d>(&'d self) -> Vec<&'d wgpu::BindGroup>{
                let ($($name, )+) = self;
                vec![$($name.get_bind_group()),+]
            }
            fn bind_group_layouts(device: &wgpu::Device) -> Vec<BindGroupLayoutWithDesc>{
                vec![$($name::create_bind_group_layout(device, None, wgpu::ShaderStages::all())),+]
            }
        }
    }
}


render_data_for_tuple!{ A }
render_data_for_tuple!{ A B }
render_data_for_tuple!{ A B C }
render_data_for_tuple!{ A B C D }
render_data_for_tuple!{ A B C D E }
render_data_for_tuple!{ A B C D E F }
render_data_for_tuple!{ A B C D E F G }
render_data_for_tuple!{ A B C D E F G H }
render_data_for_tuple!{ A B C D E F G H I }
render_data_for_tuple!{ A B C D E F G H I J }
render_data_for_tuple!{ A B C D E F G H I J K }
render_data_for_tuple!{ A B C D E F G H I J K L }

macro_rules! render_data_for_tuple_bound{
    ($($name:ident)+) => {
        #[allow(non_snake_case)]
        impl<'rd, $($name: BindGroupContent),+> RenderData for ($(&'rd Bound<$name>, )+){
            fn bind_groups<'d>(&'d self) -> Vec<&'d wgpu::BindGroup>{
                let ($($name, )+) = self;
                vec![$($name.get_bind_group()),+]
            }
            fn bind_group_layouts(device: &wgpu::Device) -> Vec<BindGroupLayoutWithDesc>{
                vec![$($name::create_bind_group_layout(device, None, wgpu::ShaderStages::all())),+]
            }
        }
    }
}

render_data_for_tuple_bound!{ A }
render_data_for_tuple_bound!{ A B }
render_data_for_tuple_bound!{ A B C }
render_data_for_tuple_bound!{ A B C D }
render_data_for_tuple_bound!{ A B C D E }
render_data_for_tuple_bound!{ A B C D E F }
render_data_for_tuple_bound!{ A B C D E F G }
render_data_for_tuple_bound!{ A B C D E F G H }
render_data_for_tuple_bound!{ A B C D E F G H I }
render_data_for_tuple_bound!{ A B C D E F G H I J }
render_data_for_tuple_bound!{ A B C D E F G H I J K }
render_data_for_tuple_bound!{ A B C D E F G H I J K L }
*/
