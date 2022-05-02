
use imgui_winit_support::WinitPlatform;
use std::ops::{Deref, DerefMut};
use crate::*;
use winit::event::*;

pub struct ImguiRenderContext<'irc, 'ui>{
    pub platform: &'irc WinitPlatform,
    pub renderer: &'irc imgui_wgpu::Renderer,
    pub ui: &'irc imgui::Ui<'ui>,
}

impl<'irc, 'ui> Deref for ImguiRenderContext<'irc, 'ui>{
    type Target = imgui::Ui<'ui>;

    fn deref(&self) -> &Self::Target {
        self.ui
    }
}

///
///
/// A context handeling all Imgui handles.
///
/// ```ignore
///
/// env_logger::init();
/// let event_loop = EventLoop::new();
///
/// let window = WindowBuilder::new()
///     .with_transparent(true)
///     .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
///     .build(&event_loop).unwrap();
///
/// let instance = wgpu::Instance::new(wgpu::Backends::all());
///
/// let mut winit = GPUContextBuilder::new()
///     .set_features_util()
///     .build_winit_context(&instance, window);
///
/// let mut imgui = ImguiContext::new(&winit);
///
/// event_loop.run(move |event, _, control_flow|{
///     let mut imgui = imgui.handle_events(&winit, &event);
///     let mut winit = winit.handle_events(&event, control_flow);
///     winit.on_redraw_encode(&event, control_flow, |winit, view, encoder, event, control_flow|{
///
///         imgui.ui(winit, encoder, view, |ui, winit, encoder|{
///             imgui::Window::new("Test")
///                 .size([100., 100.], imgui::Condition::FirstUseEver)
///                 .build(&ui, ||{
///                     ui.text("test");
///                 });
///         });
///
///         Ok(())
///     });
///
///     winit.on_resize(&event, |winit, size|{
///
///     });
/// });
/// ```
///
pub struct ImguiContext{
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub renderer: imgui_wgpu::Renderer,
}

impl ImguiContext{
    pub fn new(winit_context: &WinitContext) -> Self{

        let mut context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(
            context.io_mut(),
            &winit_context.window,
            imgui_winit_support::HiDpiMode::Default,
        );
        context.set_ini_filename(None);

        let hidpi_factor = winit_context.window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        context.fonts().add_font(&[
            imgui::FontSource::DefaultFontData{
                config: Some(imgui::FontConfig{
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }
        ]);

        let renderer_config = imgui_wgpu::RendererConfig{
            texture_format: winit_context.config.format,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(&mut context, &winit_context.device, &winit_context.queue, renderer_config);

        Self{
            renderer,
            context,
            platform,
        }
    }

    pub fn handle_events(&mut self, winit: &WinitContext, event: &Event<()>) -> UpdatedImguiContext{
            self.platform.handle_event(self.context.io_mut(), &winit.window, event);
            UpdatedImguiContext{
                imgui: self
            }
    }
}

///
/// Handle to prevent ui being called before handle_events.
/// TODO: find better name
/// ```ignore
///
/// env_logger::init();
/// let event_loop = EventLoop::new();
///
/// let window = WindowBuilder::new()
///     .with_transparent(true)
///     .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
///     .build(&event_loop).unwrap();
///
/// let instance = wgpu::Instance::new(wgpu::Backends::all());
///
/// let mut winit = GPUContextBuilder::new()
///     .set_features_util()
///     .build_winit_context(&instance, window);
///
/// let mut imgui = ImguiContext::new(&winit);
///
/// event_loop.run(move |event, _, control_flow|{
///     // This is the UpdatedImguiContext
///     let mut imgui = imgui.handle_events(&winit, &event);
///     let mut winit = winit.handle_events(&event, control_flow);
///     winit.on_redraw_encode(&event, control_flow, |winit, view, encoder, event, control_flow|{
///
///         imgui.ui(winit, encoder, view, |ui, winit, encoder|{
///             imgui::Window::new("Test")
///                 .size([100., 100.], imgui::Condition::FirstUseEver)
///                 .build(&ui, ||{
///                     ui.text("test");
///                 });
///         });
///
///         Ok(())
///     });
///
///     winit.on_resize(&event, |winit, size|{
///
///     });
/// });
/// ```
///
pub struct UpdatedImguiContext<'uic>{
    imgui: &'uic mut ImguiContext,
}

impl<'uic> UpdatedImguiContext<'uic>{

    pub fn ui<F>(&mut self, winit_context: &mut WinitContext, encoder: &mut wgpu::CommandEncoder, dst: &wgpu::TextureView, mut f: F)
        where F: FnMut(ImguiRenderContext, &mut WinitContext, &mut wgpu::CommandEncoder)
    {
        let imgui: &mut ImguiContext = self;
        imgui.platform
            .prepare_frame(imgui.context.io_mut(), &winit_context.window)
            .expect("Failed to prepare frame.");

        imgui.context.io_mut().update_delta_time(winit_context.dt);

        let ui = imgui.context.frame();

        let imgui_render_context = ImguiRenderContext{
            platform: &imgui.platform,
            renderer: &imgui.renderer,
            ui: &ui,
        };

        f(imgui_render_context, winit_context, encoder);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label: None,
            color_attachments: &[dst.color_attachment_load()],
            depth_stencil_attachment: None,
        });

        imgui.renderer.render(ui.render(), &winit_context.queue, &winit_context.device, &mut rpass)
            .expect("Rendering Failed");

        drop(rpass);
    }
}

impl<'uic> Deref for UpdatedImguiContext<'uic>{
    type Target = ImguiContext;

    fn deref(&self) -> &Self::Target {
        self.imgui
    }
}

impl<'uic> DerefMut for UpdatedImguiContext<'uic>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.imgui
    }
}
