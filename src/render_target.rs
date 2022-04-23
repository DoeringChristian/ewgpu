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

pub trait ColorAttachmentData<'cad>{
    fn color_attachments(self) -> Vec<wgpu::RenderPassColorAttachment<'cad>>;
}

impl<'cad> ColorAttachmentData<'cad> for (wgpu::RenderPassColorAttachment<'cad>, ){
    fn color_attachments(self) -> Vec<wgpu::RenderPassColorAttachment<'cad>> {
        vec![self.0]
    }
}

