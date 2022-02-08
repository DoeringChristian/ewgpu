use anyhow::*;
use super::texture;

///
/// RenderTarget should be implemented for anything that can be rendered to.
///
///
///

pub trait RenderTarget{
    fn render_pass_clear<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>>;
    fn render_pass_load<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>>;
}

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

impl RenderTarget for wgpu::TextureView{
    fn render_pass_clear<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>> {
        Ok(encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label,
            color_attachments: &[
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
            ],
            depth_stencil_attachment: None
        }))
    }
    fn render_pass_load<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>> {
        Ok(encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label,
            color_attachments: &[
                wgpu::RenderPassColorAttachment{
                    view: self,
                    resolve_target: None,
                    ops: wgpu::Operations{
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }
            ],
            depth_stencil_attachment: None
        }))
    }
}

impl RenderTarget for [&wgpu::TextureView]{
    fn render_pass_clear<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>>{
        let mut color_attachments: Vec<wgpu::RenderPassColorAttachment> = Vec::with_capacity(self.len());

        for view in self{
            color_attachments.push(
                wgpu::RenderPassColorAttachment{
                    view: *view,
                    resolve_target: None,
                    ops: wgpu::Operations{
                        load: wgpu::LoadOp::Clear( wgpu::Color{
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }
                              ),
                              store: true,
                    },
                }
            );
        }

        Ok(encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label,
            color_attachments: &&color_attachments,
            depth_stencil_attachment: None,
        }))
    }
    fn render_pass_load<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>> {
        let mut color_attachments: Vec<wgpu::RenderPassColorAttachment> = Vec::with_capacity(self.len());

        for view in self{
            color_attachments.push(
                wgpu::RenderPassColorAttachment{
                    view: *view,
                    resolve_target: None,
                    ops: wgpu::Operations{
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }
            );
        }

        Ok(encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label,
            color_attachments: &&color_attachments,
            depth_stencil_attachment: None,
        }))
    }
}

