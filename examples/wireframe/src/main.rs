use ewgpu::*;
use imgui;
use winit::{window::WindowBuilder, event_loop::EventLoop};
use wireframe::*;
use camera::*;

mod wireframe;
mod camera;

extern crate nalgebra_glm as glm;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Consts{
    pub color: [f32; 4],
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_transparent(true)
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop).unwrap();

    let mut gpu = GPUContextBuilder::new()
        .set_features_util()
        .set_limits(wgpu::Limits{
            max_push_constant_size: 128,
            ..Default::default()
        })
        .build_winit_context(window);

    let mut imgui = ImguiContext::new(&gpu);

    let vertices = [
        WireframeVert::new(
            [0.9, 0., -1.],
            [1., 0., 0., 1.],
            1.,
        ),
        WireframeVert::new(
            [-0.9, 0., -2.],
            [0., 1., 0., 1.],
            1.,
        ),
        WireframeVert::new(
            [-0.9, 0.9, -1.],
            [0., 0., 1., 1.],
            1.,
        ),
    ];

    let indices = [0, 1, 1, 2, 0, 2];
    let wireframe = GPUWireframe::new(&gpu.device, &vertices, &indices, 3.);

    let mut wireframe_renderer = WireframeRenderer::new(&gpu.device, gpu.config.format);


    event_loop.run(move |event, _, control_flow|{
        let mut imgui = imgui.handle_events(&gpu, &event);
        let mut gpu = gpu.handle_events(&event, control_flow);
        gpu.on_redraw_encode(&event, control_flow, |gpu, view, encoder, event, control_flow|{
            {
                let norm_size = glm::vec2(gpu.size.width as f32, gpu.size.height as f32).normalize();
                let mvp: glm::Mat4 = glm::perspective_fov_zo(std::f32::consts::PI / 2., norm_size.x, norm_size.y, 0.01, 100.).into();
                let mvp = glm::perspective_zo(gpu.size.width as f32 / gpu.size.height as f32, std::f32::consts::PI / 2., 0.01, 100.);
                //let mvp = glm::Mat4::identity();
                let camera = Camera{
                    mvp: mvp.into(),
                };
                wireframe_renderer.update(&wireframe, &camera, [gpu.size.width as f32, gpu.size.height as f32], &gpu.queue, encoder);
                wireframe_renderer.render(&wireframe, encoder, view);
            }

            imgui.ui(gpu, encoder, view, |ui, gpu, encoder|{
                imgui::Window::new("Test")
                    .size([100., 100.], imgui::Condition::FirstUseEver)
                    .build(&ui, ||{
                        ui.text("test");
                    });
            });

            Ok(())
        });

        gpu.on_resize(&event, |winit, size|{

        });
    });
}
