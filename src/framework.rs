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

pub trait State{
    fn new(app: &mut AppState) -> Self;
    fn render(&mut self, app: &mut AppState, control_flow: &mut ControlFlow, dst: wgpu::TextureView) -> Result<(), wgpu::SurfaceError>{Ok(())}
    fn pre_render(&mut self, app: &mut AppState, control_flow: &mut ControlFlow) -> Result<(), wgpu::SurfaceError>{Ok(())}
    fn input(&mut self, event: &WindowEvent) -> bool{false}
    fn cursor_moved(&mut self, fstate: &mut AppState, device_id: &winit::event::DeviceId, position: &winit::dpi::PhysicalPosition<f64>){}
    fn device_event(&mut self, fstate: &mut AppState, device_id: &winit::event::DeviceId, device_event: &DeviceEvent){}
    fn resize(&mut self, fstate: &mut AppState, new_size: winit::dpi::PhysicalSize<u32>){}
}

pub struct AppState{
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub frame_count: usize,
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
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty()
                    .union(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
                    .union(wgpu::Features::VERTEX_WRITABLE_STORAGE)
                    .union(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS),
                limits: wgpu::Limits::default(),
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

        Self{
            surface,
            device,
            queue,
            config,
            size,
            window,
            frame_count: 0,
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

pub struct Framework<S: State>{
    app: AppState,
    state: S,
    event_loop: EventLoop<()>,
}

impl<S: 'static +  State> Framework<S>{

    pub fn new(size: [u32; 2]) -> Self{
        env_logger::init();

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(size[0], size[1]))
            .build(&event_loop).unwrap();

        let mut app = pollster::block_on(AppState::new(window));

        let state = S::new(&mut app);

        Self{
            app,
            state,
            event_loop,
        }
    }

    pub fn run(mut self){
        self.event_loop.run(move |event, _, control_flow|{
            match event{
                Event::WindowEvent{
                    ref event,
                    window_id,
                } if window_id == self.app.window.id() => if !self.state.input(event){
                    match event{
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.app.resize(*physical_size);
                            self.state.resize(&mut self.app, *physical_size);
                        },

                        WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                            self.app.resize(**new_inner_size);
                            self.state.resize(&mut self.app, **new_inner_size);
                        },
                        WindowEvent::CursorMoved{device_id, position, ..} => {
                            self.state.cursor_moved(&mut self.app, device_id, position);
                        }
                        _ => {},
                    }
                },

                Event::RedrawRequested(window_id) if window_id == self.app.window.id() => {
                    if self.app.frame_count == 0{
                        match self.state.pre_render(&mut self.app, control_flow){
                            Ok(_) => {}

                            Err(wgpu::SurfaceError::Lost) => self.app.resize(self.app.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    let output = match self.app.surface.get_current_texture(){
                        Ok(o) => {o},
                        Err(e) => {eprintln!("{:?}", e); return},
                    };
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    match self.state.render(&mut self.app, control_flow, view){
                        Ok(_) => {}

                        Err(wgpu::SurfaceError::Lost) => self.app.resize(self.app.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                        Err(e) => eprintln!("{:?}", e),
                    }
                    output.present();
                    self.app.frame_count += 1;
                },
                Event::DeviceEvent{device_id, event} => {
                    self.state.device_event(&mut self.app, &device_id, &event);
                }

                Event::MainEventsCleared => {
                    self.app.window.request_redraw();
                },
                _ => {}
            }
        });
    }
}
