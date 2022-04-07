use std::ops::Deref;

use crate::*;

pub trait RenderData<'rp>{
    fn bind_groups(self) -> Vec<&'rp wgpu::BindGroup>;
}

macro_rules! render_data_for_tuple{
    ($($name:ident)+) => {
        impl<'rp, $($name: GetBindGroup),+> RenderData<'rp> for ($(&'rp $name, )+){
            fn bind_groups(self) -> Vec<&'rp wgpu::BindGroup>{
                let ($($name, )+) = self;
                vec![$($name.get_bind_group()),+]
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
