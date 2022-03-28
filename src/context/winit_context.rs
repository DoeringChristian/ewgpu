
use crate::*;
use winit::{
    event::*,
    event_loop::ControlFlow,
    window::Window,
};

use std::ops::Deref;
use std::ops::DerefMut;

pub trait BuildWinitContext{
    fn build_winit_context(self, window: Window) -> WinitContext;
}

impl<'gcb> BuildWinitContext for GPUContextBuilder<'gcb>{
    fn build_winit_context(self, window: Window) -> WinitContext {
        let winit_context_builder: WinitContextBuilder = self.into();
        winit_context_builder.build(window)
    }
}

impl<'wcb> From<GPUContextBuilder<'wcb>> for WinitContextBuilder<'wcb>{
    fn from(gpu_context_builder: GPUContextBuilder<'wcb>) -> Self {
        WinitContextBuilder{
            gpu_context_builder
        }
    }
}

pub struct WinitContextBuilder<'wcb>{
    gpu_context_builder: GPUContextBuilder<'wcb>,
}

impl<'wcb> WinitContextBuilder<'wcb>{
    pub fn build(self, window: Window) -> WinitContext{

        let instance = wgpu::Instance::new(self.gpu_context_builder.backends);

        let surface = unsafe{instance.create_surface(&window)};

        let size = window.inner_size();

        let gpu_context = self.gpu_context_builder
            .set_compatible_surface(Some(&surface))
            .build_with_instance(instance);

        let config = wgpu::SurfaceConfiguration{
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&gpu_context.adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&gpu_context.device, &config);

        WinitContext{
            gpu_context,
            surface,
            config,
            size,
            window,
        }
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
    // TODO: unimplement
    pub fn new(instance: wgpu::Instance, window: Window) -> Self{
        let surface = unsafe{instance.create_surface(&window)};

        let size = window.inner_size();

        let gpu_context = GPUContext::new(instance, Some(&surface));
        //let gpu_context = gpu_context_builder.set_compatible_surface(Some(&surface)).build(instance);

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
    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
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


    ///
    /// Has to be called in the event_loop before all other functions.
    /// TODO: return handle that can restrict callability on on_redraw etc.
    ///
    /// ```ignore
    ///
    /// env_logger::init();
    /// let event_loop = EventLoop::new();
    ///
    /// let window = WindowBuilder::new()
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
    ///     imgui.handle_events(&winit, &event);
    ///     let mut winit = winit.handle_events(&event, control_flow);
    ///     winit.on_resize(&event, |winit, size|{
    ///
    ///     });
    /// });
    ///
    /// ```
    ///
    pub fn handle_events(&mut self, event: &Event<()>, control_flow: &mut ControlFlow) -> UpdatedWinitContext
    {
        match *event{
            Event::WindowEvent{
                ref event,
                window_id,
            } if window_id == self.window.id() => {
                match event{
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        self.resize(*physical_size);
                        //resize(self, *physical_size);
                    },
                    WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                        self.resize(**new_inner_size);
                        //resize(self, **new_inner_size);
                    },
                    _ => {},
                }
            },

            Event::MainEventsCleared => {
                self.window.request_redraw();
            },
            _ => {}
        }
        UpdatedWinitContext{
            winit: self
        }
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


///
/// This is a handle to prevent call to any functions that need window events to be handled
/// before they have been handled.
///
pub struct UpdatedWinitContext<'uwc>{
    winit: &'uwc mut WinitContext,
}

impl<'uwc> Deref for UpdatedWinitContext<'uwc>{
    type Target = WinitContext;

    fn deref(&self) -> &Self::Target{
        &self.winit
    }
}

impl<'uwc> DerefMut for UpdatedWinitContext<'uwc>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.winit
    }
}


impl<'uwc> UpdatedWinitContext<'uwc>{
    ///
    /// Can be used to render to the surface.
    ///
    /// ```ignore
    ///
    /// env_logger::init();
    /// let event_loop = EventLoop::new();
    ///
    /// let window = WindowBuilder::new()
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
    ///     imgui.handle_events(&winit, &event);
    ///     let mut winit = winit.handle_events(&event, control_flow);
    ///     winit.on_redraw(&event, |winit, event|{
    ///         winit.encode(control_flow, |winit, view, encoder, control_flow|{
    ///
    ///         });
    ///     });
    /// });
    ///
    /// ```
    ///
    pub fn encode<F>(&mut self, control_flow: &mut ControlFlow, mut f: F)
        where F: FnMut(&mut Self, &wgpu::TextureView, &mut wgpu::CommandEncoder, &mut ControlFlow) -> Result<(), wgpu::SurfaceError>
    {
        let output = match self.surface.get_current_texture(){
            Ok(o) => {o},
            Err(e) => {eprintln!("{:?}", e); return},
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("imgui_encoder")});

        // Call render function 
        let size = self.size;

        match f(self, &view, &mut encoder, control_flow){
            Ok(_) => {}

            Err(wgpu::SurfaceError::Lost) => self.resize(size),
            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

            Err(e) => eprintln!("{:?}", e),
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        self.update();
    }

    ///
    /// Called if in this run there has been a request_redraw event.
    ///
    /// ```ignore
    ///
    /// env_logger::init();
    /// let event_loop = EventLoop::new();
    ///
    /// let window = WindowBuilder::new()
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
    ///     imgui.handle_events(&winit, &event);
    ///     let mut winit = winit.handle_events(&event, control_flow);
    ///     winit.on_redraw(&event, |winit, event|{
    ///         // Do something.
    ///     });
    /// });
    ///
    /// ```
    ///
    pub fn on_redraw<RD>(&mut self, event: &Event<()>, mut f: RD)
        where RD: FnMut(&mut Self, &Event<()>)
    {
        match *event{
            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                f(self, event);
            }
            _ => {}
        }
    }

    ///
    /// Combines on_redraw and encode.
    ///
    /// ```ignore
    ///
    /// env_logger::init();
    /// let event_loop = EventLoop::new();
    ///
    /// let window = WindowBuilder::new()
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
    ///     imgui.handle_events(&winit, &event);
    ///     let mut winit = winit.handle_events(&event, control_flow);
    ///     winit.on_redraw_encode(&event, control_flow, |winit, view, encoder, event, control_flow|{
    ///
    ///        Ok(())
    ///     });
    /// });
    ///
    /// ```
    ///
    pub fn on_redraw_encode<E>(&mut self, event: &Event<()>, control_flow: &mut ControlFlow, mut f: E)
        where E: FnMut(&mut Self, &wgpu::TextureView, &mut wgpu::CommandEncoder, &Event<()>, &mut ControlFlow) -> Result<(), wgpu::SurfaceError>
    {
        self.on_redraw(event, |winit, event|{
            winit.encode(control_flow, |winit, view, encoder, control_flow|{
                f(winit, view, encoder, event, control_flow)
            });
        });
    }

    ///
    /// Called on resize in this loop.
    ///
    /// ```ignore
    ///
    /// env_logger::init();
    /// let event_loop = EventLoop::new();
    ///
    /// let window = WindowBuilder::new()
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
    ///     imgui.handle_events(&winit, &event);
    ///     let mut winit = winit.handle_events(&event, control_flow);
    ///     winit.on_resize(&event, |winit, size|{
    ///         
    ///     });
    /// });
    ///
    /// ```
    ///
    pub fn on_resize<R>(&mut self, event: &Event<()>, mut f: R)
        where R: FnMut(&mut Self, winit::dpi::PhysicalSize<u32>)
    {
        match *event{
            Event::WindowEvent{
                ref event,
                window_id,
            } if window_id == self.window.id() => {
                match event{
                    WindowEvent::Resized(physical_size) => {
                        f(self, *physical_size);
                    },
                    WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                        f(self, **new_inner_size);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
