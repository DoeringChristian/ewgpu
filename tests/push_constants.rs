use ewgpu::*;

#[test]
fn push_const_compute(){
    use crate::*;

    let mut gpu = GPUContextBuilder::new()
        .set_features_util()
        .set_limits(wgpu::Limits{
            max_push_constant_size: 128,
            ..Default::default()
        })
        .build();
    let cshader = ComputeShader::from_src(&gpu.device, "
            #version 460
            #if COMPUTE_SHADER

            layout(set = 0, binding = 0) buffer OutBuffer{
                uint out_buf[];
            };
            layout(push_constant) uniform PushConstants{
                uint push_const;
            };

            void main(){
                uint i = gl_GlobalInvocationID.x;

                out_buf[i] = push_const;
            }
            #endif
            ", None).unwrap();
            let out_buf = BufferBuilder::<u32>::new()
                .storage().read()
                .build_empty(&gpu.device, 1);
    let out_buf = BindGroup::new(out_buf, &gpu.device);

    /*
       let layout = PipelineLayoutBuilder::new()
       .push_bind_group(&BindGroup::<Buffer<u32>>::create_bind_group_layout(&gpu.device, None))
       .push_const_layout(u32::push_const_layout(wgpu::ShaderStages::COMPUTE))
       .build(&gpu.device, None);
       */
    let layout = pipeline_layout!(&gpu.device, 
        bind_groups: {
            buffer1: BindGroup::<Buffer<u32>>,
        },
        push_constants: {
            u32 => wgpu::ShaderStages::COMPUTE,
        }
    );

    let cpipeline = ComputePipelineBuilder::new(&cshader)
        .set_layout(&layout)
        .build(&gpu.device);

    gpu.encode(|gpu, encoder|{
        {
            let mut cpass = ComputePass::new(encoder, None);

            let mut cpass_ppl = cpass.set_pipeline(&cpipeline);

            cpass_ppl.set_bind_group(0, &out_buf, &[]);
            cpass_ppl.set_push_const(0, &(3 as u32));
            cpass_ppl.dispatch(out_buf.len() as u32, 1, 1);
        }
    });
    assert_eq!(out_buf.slice(..).map_blocking(&gpu.device).as_ref(), [3]);
}
