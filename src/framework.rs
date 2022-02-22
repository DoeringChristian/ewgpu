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

/*
pub trait State{
    fn new(app: &mut AppState) -> Self;
    fn init_imgui(&mut self, app: &mut AppState, imgui: &mut ImguiState){}
    fn render(&mut self, app: &mut AppState, control_flow: &mut ControlFlow, dst: &wgpu::TextureView) -> Result<(), wgpu::SurfaceError>{Ok(())}
    fn render_imgui(&mut self, app: &mut AppState, control_flow: &mut ControlFlow, imgui: ImguiRenderContext<'_, '_>){}
    fn pre_render(&mut self, app: &mut AppState, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError>{Ok(())}
    fn input(&mut self, event: &WindowEvent) -> bool{false}
    fn cursor_moved(&mut self, fstate: &mut AppState, device_id: &winit::event::DeviceId, position: &winit::dpi::PhysicalPosition<f64>){}
    fn device_event(&mut self, fstate: &mut AppState, device_id: &winit::event::DeviceId, device_event: &DeviceEvent){}
    fn resize(&mut self, fstate: &mut AppState, new_size: winit::dpi::PhysicalSize<u32>){}
}
*/

#[allow(unused)]
pub trait State<C: Context>{
    fn new(context: &mut C) -> Self;
    fn render(&mut self, context: &mut C, dst: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError>{Ok(())}
    fn render_imgui(&mut self, context: &mut WinitContext, imgui: ImguiRenderContext<'_, '_>, control_flow: &mut ControlFlow){}
    fn input(&mut self, event: &WindowEvent) -> bool{false}
    fn cursor_moved(&mut self, context: &mut C, device_id: &winit::event::DeviceId, position: &winit::dpi::PhysicalPosition<f64>){}
    fn device_event(&mut self, context: &mut C, device_id: &winit::event::DeviceId, device_event: &DeviceEvent){}
    fn resize(&mut self, context: &mut C, new_size: winit::dpi::PhysicalSize<u32>){}
}

#[allow(unused)]
pub trait Context{
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){}
    fn update(&mut self);
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
}

impl Context for GPUContext{
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
    pub fn new(window: Window) -> Self{
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe{instance.create_surface(&window)};

        let size = window.inner_size();

        let gpu_context = GPUContext::new(&instance, Some(&surface));

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
}

impl Context for WinitContext{
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
    pub winit_context: WinitContext,
    pub imgui_context: RefCell<imgui::Context>,
    pub platform: RefCell<WinitPlatform>,
    pub renderer: RefCell<imgui_wgpu::Renderer>,
}

impl ImguiContext{
    pub fn new(window: Window) -> Self{

        let winit_context = WinitContext::new(window);

        let mut imgui_context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
        platform.attach_window(
            imgui_context.io_mut(),
            &winit_context.window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui_context.set_ini_filename(None);

        let hidpi_factor = winit_context.window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui_context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui_context.fonts().add_font(&[
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

        let renderer = imgui_wgpu::Renderer::new(&mut imgui_context, &winit_context.device, &winit_context.queue, renderer_config);

        let imgui_context = RefCell::new(imgui_context);
        let renderer = RefCell::new(renderer);
        let platform = RefCell::new(platform);

        Self{
            winit_context,
            imgui_context,
            platform,
            renderer,
        }
    }

    pub fn ui<F>(&mut self, f: F, encoder: &mut wgpu::CommandEncoder, dst: &wgpu::TextureView)
        where F: Fn(&imgui::Ui, &mut imgui_wgpu::Renderer, &WinitPlatform, &mut WinitContext)
    {
        let mut renderer = self.renderer.borrow_mut();
        let mut context = self.imgui_context.borrow_mut();
        let platform = self.platform.borrow();
        {
            platform
                .prepare_frame(context.io_mut(), &self.window)
                .expect("Failed to prepare frame.");

            //context.io_mut().update_delta_time(self.dt);

            let ui = context.frame();

            f(&ui, &mut renderer, &platform, &mut self.winit_context);

            let mut rpass = RenderPassBuilder::new()
                .push_color_attachment(dst.color_attachment_load())
                .begin(encoder, None);

            renderer.render(ui.render(), &self.queue, &self.device, &mut rpass.render_pass)
                .expect("Rendering Failed");

            drop(rpass);
        }

    }
}

impl Context for ImguiContext{
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
        self.winit_context.resize(new_size)
    }
    fn update(&mut self) {
        self.winit_context.update();
    }
}

impl Deref for ImguiContext{
    type Target = WinitContext;

    fn deref(&self) -> &Self::Target {
        &self.winit_context
    }
}

impl DerefMut for ImguiContext{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.winit_context
    }
}

pub struct AppState{
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub frame_count: usize,
    pub time: Instant,
}

pub struct ImguiState{
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub renderer: imgui_wgpu::Renderer,
}

pub struct ImguiRenderContext<'irc, 'ui>{
    pub platform: &'irc WinitPlatform,
    pub renderer: &'irc imgui_wgpu::Renderer,
    pub ui: &'irc imgui::Ui<'ui>,
}

impl AppState{
    pub async fn new(window: Window) -> Self{
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe{instance.create_surface(&window)};
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions{
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
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
        let config = wgpu::SurfaceConfiguration{
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let time = Instant::now();

        Self{
            surface,
            device,
            queue,
            config,
            size,
            window,
            frame_count: 0,
            time,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

}

pub struct Framework<C: Context, S: State<C>>{
    context: C,
    state: S,
    event_loop: EventLoop<()>,
}

impl<C: Context, S: 'static + State<C>> Framework<C, S>{
}

impl<S: 'static + State<WinitContext>> Framework<WinitContext, S>{
    pub fn new(window_builder: WindowBuilder) -> Self{
        env_logger::init();
        let event_loop = EventLoop::new();

        let window = window_builder.build(&event_loop).unwrap();

        let mut context = WinitContext::new(window);

        let state = S::new(&mut context);

        Self{
            context,
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
                } if window_id == self.context.window.id() => if !self.state.input(event){
                    match event{
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.context.resize(*physical_size);
                            self.state.resize(&mut self.context, *physical_size);
                        },

                        WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                            self.context.resize(**new_inner_size);
                            self.state.resize(&mut self.context, **new_inner_size);
                        },
                        WindowEvent::CursorMoved{device_id, position, ..} => {
                            self.state.cursor_moved(&mut self.context, device_id, position);
                        }
                        _ => {},
                    }
                },

                Event::RedrawRequested(window_id) if window_id == self.context.window.id() => {
                    let output = match self.context.surface.get_current_texture(){
                        Ok(o) => {o},
                        Err(e) => {eprintln!("{:?}", e); return},
                    };
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("encoder")});

                    // Call render function 
                    match self.state.render(&mut self.context, &view, &mut encoder, control_flow){
                        Ok(_) => {}

                        Err(wgpu::SurfaceError::Lost) => self.context.resize(self.context.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                        Err(e) => eprintln!("{:?}", e),
                    }

                    self.context.queue.submit(Some(encoder.finish()));
                    output.present();
                    self.context.update();
                },
                Event::DeviceEvent{device_id, ref event} => {
                    self.state.device_event(&mut self.context, &device_id, &event);
                }

                Event::MainEventsCleared => {
                    self.context.window.request_redraw();
                },
                _ => {}
            }
        });
    }
} 

impl<S: 'static + State<ImguiContext>> Framework<ImguiContext, S>{
    pub fn new(window_builder: WindowBuilder) -> Self{
        env_logger::init();
        let event_loop = EventLoop::new();

        let window = window_builder.build(&event_loop).unwrap();

        let mut context = ImguiContext::new(window);

        let state = S::new(&mut context);

        Self{
            context,
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
                } if window_id == self.context.window.id() => if !self.state.input(event){
                    match event{
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.context.resize(*physical_size);
                            self.state.resize(&mut self.context, *physical_size);
                        },

                        WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                            self.context.resize(**new_inner_size);
                            self.state.resize(&mut self.context, **new_inner_size);
                        },
                        WindowEvent::CursorMoved{device_id, position, ..} => {
                            self.state.cursor_moved(&mut self.context, device_id, position);
                        }
                        _ => {},
                    }
                },

                Event::RedrawRequested(window_id) if window_id == self.context.window.id() => {
                    let output = match self.context.surface.get_current_texture(){
                        Ok(o) => {o},
                        Err(e) => {eprintln!("{:?}", e); return},
                    };
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("imgui_encoder")});

                    self.context.imgui_context.borrow_mut().io_mut().update_delta_time(self.context.dt);

                    // Call render function 
                    match self.state.render(&mut self.context, &view, &mut encoder, control_flow){
                        Ok(_) => {}

                        Err(wgpu::SurfaceError::Lost) => self.context.resize(self.context.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                        Err(e) => eprintln!("{:?}", e),
                    }

                    {

                        let mut imgui_context = self.context.imgui_context.borrow_mut();
                        let mut renderer = self.context.renderer.borrow_mut();
                        let platform = self.context.platform.borrow_mut();
                        {
                            platform
                                .prepare_frame(imgui_context.io_mut(), &self.context.window)
                                .expect("Failed to prepare frame.");

                            let ui = imgui_context.frame();

                            let render_context = ImguiRenderContext{
                                platform: &*platform,
                                renderer: &*renderer,
                                ui: &ui,
                            };

                            self.state.render_imgui(&mut self.context.winit_context, render_context, control_flow);

                            let mut rpass = RenderPassBuilder::new()
                                .push_color_attachment(view.color_attachment_load())
                                .begin(&mut encoder, None);

                            renderer
                                .render(ui.render(), &self.context.queue, &self.context.device, &mut rpass.render_pass)
                                .expect("Rendering Failed");

                            drop(rpass);
                        }
                    }

                    self.context.queue.submit(Some(encoder.finish()));
                    output.present();
                },
                Event::DeviceEvent{device_id, ref event} => {
                    self.state.device_event(&mut self.context, &device_id, &event);
                }

                Event::MainEventsCleared => {
                    self.context.window.request_redraw();
                },
                _ => {}
            }
            let mut imgui_context = self.context.imgui_context.borrow_mut();
            self.context.platform.borrow_mut().handle_event(imgui_context.io_mut(), &self.context.window, &event);
        });
    }
}

