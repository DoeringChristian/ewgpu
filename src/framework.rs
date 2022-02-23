use imgui_winit_support::WinitPlatform;
use winit::{platform::unix::WindowBuilderExtUnix, dpi::PhysicalSize};
#[allow(unused)]
use winit::{
    event::*,
    event_loop::{
        ControlFlow,
        EventLoop,
    },
    window::{
        Window,
        WindowBuilder,
    },
};

use std::{time::{Instant, Duration}, ops::{Deref, DerefMut}, marker::PhantomData, cell::RefCell};
use super::*;

#[allow(unused)]
pub trait State{
    fn new(winit: &mut WinitContext, imgui: &mut ImguiContext) -> Self;
    fn render(&mut self, winit: &mut WinitContext, imgui: &mut ImguiContext, dst: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError>{Ok(())}
    fn input(&mut self, event: &WindowEvent) -> bool{false}
    fn cursor_moved(&mut self, context: &mut ImguiContext, device_id: &winit::event::DeviceId, position: &winit::dpi::PhysicalPosition<f64>){}
    fn device_event(&mut self, context: &mut ImguiContext, device_id: &winit::event::DeviceId, device_event: &DeviceEvent){}
    fn resize(&mut self, context: &mut ImguiContext, new_size: winit::dpi::PhysicalSize<u32>){}
}

pub struct GPUContext{
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub time: Instant,
    pub dt: Duration,
}

impl GPUContext{
    pub fn new(instance: &wgpu::Instance, surface: Option<&wgpu::Surface>) -> Self{
        pollster::block_on(Self::new_async(instance, surface))
    }

    pub async fn new_async(instance: &wgpu::Instance, surface: Option<&wgpu::Surface>) -> Self{
        //let instance = wgpu::Instance::new(wgpu::Backends::all());

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions{
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: surface,
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let limits = wgpu::Limits{
            max_push_constant_size: 128,
            ..Default::default()
        };

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty()
                    .union(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
                    .union(wgpu::Features::VERTEX_WRITABLE_STORAGE)
                    .union(wgpu::Features::PUSH_CONSTANTS)
                    .union(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS),
                limits,
                label: None,
            },
            None,
        ).await.unwrap();

        // DT is initialized with 1 second for first frame
        Self{
            device,
            queue,
            adapter,
            time: Instant::now(),
            dt: Duration::from_secs(1),
        }
    }
    fn update(&mut self) {
        let time = Instant::now();
        self.dt = time - self.time;
        self.time = time;
    }
}

pub struct WinitContext{
    pub gpu_context: GPUContext,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
}

impl WinitContext{
    pub fn new(instance: &wgpu::Instance, window: Window) -> Self{
        let surface = unsafe{instance.create_surface(&window)};

        let size = window.inner_size();

        let gpu_context = GPUContext::new(instance, Some(&surface));

        let config = wgpu::SurfaceConfiguration{
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&gpu_context.adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&gpu_context.device, &config);

        Self{
            gpu_context,
            surface,
            config,
            size,
            window,
        }
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self) {
        self.gpu_context.update();
    }
}

impl Deref for WinitContext{
    type Target = GPUContext;

    fn deref(&self) -> &Self::Target {
        &self.gpu_context
    }
}

impl<'w> DerefMut for WinitContext{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gpu_context
    }
}

pub struct ImguiContext{
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub renderer: imgui_wgpu::Renderer,
}

pub struct ImguiInternal{
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

    pub fn ui<F>(&mut self, winit_context: &mut WinitContext, encoder: &mut wgpu::CommandEncoder, dst: &wgpu::TextureView, f: F)
        where F: Fn(ImguiRenderContext, &mut WinitContext)
    {
        self.platform
            .prepare_frame(self.context.io_mut(), &winit_context.window)
            .expect("Failed to prepare frame.");

        self.context.io_mut().update_delta_time(winit_context.dt);

        let ui = self.context.frame();

        let imgui_render_context = ImguiRenderContext{
            platform: &self.platform,
            renderer: &self.renderer,
            ui: &ui,
        };

        f(imgui_render_context, winit_context);

        let mut rpass = RenderPassBuilder::new()
            .push_color_attachment(dst.color_attachment_load())
            .begin(encoder, None);

        self.renderer.render(ui.render(), &winit_context.queue, &winit_context.device, &mut rpass.render_pass)
            .expect("Rendering Failed");

        drop(rpass);
    }
}


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

pub struct Framework<S: State>{
    pub imgui: ImguiContext,
    pub winit: WinitContext,
    pub instance: wgpu::Instance,
    pub state: S,
    pub event_loop: EventLoop<()>,
}

impl<S: 'static + State> Framework<S>{
    pub fn new(window_builder: WindowBuilder) -> Self{
        env_logger::init();
        let event_loop = EventLoop::new();

        let window = window_builder.build(&event_loop).unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let mut winit = WinitContext::new(&instance, window);
        let mut imgui = ImguiContext::new(&winit);

        let state = S::new(&mut winit, &mut imgui);

        Self{
            instance,
            imgui,
            winit,
            event_loop,
            state,
        }
    }

    pub fn run(mut self){
        self.event_loop.run(move |event, _, control_flow|{
            match event{
                Event::WindowEvent{
                    ref event,
                    window_id,
                } if window_id == self.winit.window.id() => if !self.state.input(event){
                    match event{
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.winit.resize(*physical_size);
                            self.state.resize(&mut self.imgui, *physical_size);
                        },

                        WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                            self.winit.resize(**new_inner_size);
                            self.state.resize(&mut self.imgui, **new_inner_size);
                        },
                        WindowEvent::CursorMoved{device_id, position, ..} => {
                            self.state.cursor_moved(&mut self.imgui, device_id, position);
                        }
                        _ => {},
                    }
                },

                Event::RedrawRequested(window_id) if window_id == self.winit.window.id() => {
                    let output = match self.winit.surface.get_current_texture(){
                        Ok(o) => {o},
                        Err(e) => {eprintln!("{:?}", e); return},
                    };
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = self.winit.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("imgui_encoder")});

                    // Call render function 
                    match self.state.render(&mut self.winit, &mut self.imgui, &view, &mut encoder, control_flow){
                        Ok(_) => {}

                        Err(wgpu::SurfaceError::Lost) => self.winit.resize(self.winit.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                        Err(e) => eprintln!("{:?}", e),
                    }

                    self.winit.queue.submit(Some(encoder.finish()));
                    output.present();
                    self.winit.update();
                },
                Event::DeviceEvent{device_id, ref event} => {
                    self.state.device_event(&mut self.imgui, &device_id, &event);
                }

                Event::MainEventsCleared => {
                    self.winit.window.request_redraw();
                },
                _ => {}
            }
            self.imgui.platform.handle_event(self.imgui.context.io_mut(), &self.winit.window, &event);
        });
    }
}


