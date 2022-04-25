///
/// Can be attached as the ColorAttachment of a RenderPass
///
pub trait ColorAttachment{
    fn color_attachment_clear(&self) -> wgpu::RenderPassColorAttachment;
    fn color_attachment_clear_with(&self, color: wgpu::Color) -> wgpu::RenderPassColorAttachment;
    fn color_attachment_load(&self) -> wgpu::RenderPassColorAttachment;
}

impl ColorAttachment for wgpu::TextureView{
    fn color_attachment_clear(&self) -> wgpu::RenderPassColorAttachment {
        wgpu::RenderPassColorAttachment{
            view: self,
            resolve_target: None,
            ops: wgpu::Operations{
                load: wgpu::LoadOp::Clear(wgpu::Color{
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }),
                store: true,
            },
        }
    }

    fn color_attachment_clear_with(&self, color: wgpu::Color) -> wgpu::RenderPassColorAttachment{
        wgpu::RenderPassColorAttachment{
            view: self,
            resolve_target: None,
            ops: wgpu::Operations{
                load: wgpu::LoadOp::Clear(color),
                store: true,
            },
        }
    }

    fn color_attachment_load(&self) -> wgpu::RenderPassColorAttachment{
        wgpu::RenderPassColorAttachment{
            view: self,
            resolve_target: None,
            ops: wgpu::Operations{
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }
    }
}

pub trait ColorAttachmentData{
    fn color_attachments<'c>(&'c self) -> Vec<wgpu::RenderPassColorAttachment<'c>>;
}

impl<'cad, const N: usize> ColorAttachmentData for [wgpu::RenderPassColorAttachment<'cad>; N]{
    fn color_attachments<'c>(&'c self) -> Vec<wgpu::RenderPassColorAttachment<'c>> {
        let mut v = Vec::with_capacity(N);
        v.extend_from_slice(self);
        v
    }
}

// TODO: Constant texture format.
pub struct RPassColorAttachment<'c, const F: u32>{
    attachment: wgpu::RenderPassColorAttachment<'c>,
}
