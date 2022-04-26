use egui::FontDefinitions;
use egui_wgpu_backend::RenderPass;
use egui_winit_platform::{Platform, PlatformDescriptor};
use crate::*;
use epi::*;

pub struct EguiContext{
    platform: Platform,
    rpass: RenderPass,
}

impl EguiContext{
    pub fn new(winit_context: &WinitContext) -> Self{
        let mut platform = Platform::new(PlatformDescriptor{
            physical_width: winit_context.size.width,
            physical_height: winit_context.size.height,
            scale_factor: winit_context.window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let mut rpass = RenderPass::new(&winit_context.device, winit_context.config.format, 1);

        Self{
            platform,
            rpass,
        }
    }

    /*
    pub fn handle_events(&mut self, winit: &WinitContext, event: &Event<()>) -> UpdatedEguiContext{
        self.platform.handle_event(event);
        UpdatedEguiContext{
            egui: self
        }
    }
    */
}

pub struct EguiRenderContext<'erc>{
    egui: &'erc mut UpdatedEguiContext<'erc>,
}

pub struct UpdatedEguiContext<'uec>{
    egui: &'uec mut EguiContext,
}

impl<'uec> UpdatedEguiContext<'uec>{
    pub fn ui<F>(&mut self, winit_context: &mut WinitContext, encoder: &mut wgpu::CommandEncoder, dst: &wgpu::TextureView, mut f: F)
        where F: FnMut(EguiRenderContext, &mut WinitContext, &mut wgpu::CommandEncoder){
            /*
            self.egui.platform.begin_frame();
            let app_output = epi::backend::AppOutput::default();
            let mut frame = epi::Frame::new(epi::backend::FrameData{
                info: epi::IntegrationInfo{
                    name: "frame",
                    web_info: None,
                    cpu_usage: winit_context.dt,
                    native_pixels_per_point: Some(winit_context.window.scale_factor() as _),
                    prefer_dark_mode: None,
                },
                output: app_output,
                repaint_signal: 
            })
            */

    }
}
