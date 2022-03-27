
use crate::*;
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

use std::ops::Deref;
use std::ops::DerefMut;

pub trait BuildWinitContext{
    fn build_winit_context(self, instance: &wgpu::Instance, window: Window) -> WinitContext;
}

impl<'gcb> BuildWinitContext for GPUContextBuilder<'gcb>{
    fn build_winit_context(self, instance: &wgpu::Instance, window: Window) -> WinitContext {
        let winit_context_builder: WinitContextBuilder = self.into();
        winit_context_builder.build(instance, window)
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
    pub fn build(self, instance: &wgpu::Instance, window: Window) -> WinitContext{
        let surface = unsafe{instance.create_surface(&window)};

        let size = window.inner_size();

        let gpu_context = self.gpu_context_builder
            .set_compatible_surface(Some(&surface))
            .build(instance);

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
    pub fn new(instance: &wgpu::Instance, window: Window) -> Self{
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

        match f(self, &view, &mut encoder, control_flow){
            Ok(_) => {}

            Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

            Err(e) => eprintln!("{:?}", e),
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        self.update();
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
