
use std::time::{Instant, Duration};
use crate::*;

///
/// ```rust
/// use ewgpu::*;
///
/// let instance = wgpu::Instance::new(wgpu::Backends::all());
///
/// let mut gpu = GPUContextBuilder::new()
///     .enable_feature(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
///     .enable_feature(wgpu::Features::VERTEX_WRITABLE_STORAGE)
///     .enable_feature(wgpu::Features::PUSH_CONSTANTS)
///     .enable_feature(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
///     .enable_feature(wgpu::Features::POLYGON_MODE_LINE)
///     .set_limits(wgpu::Limits{
///         max_push_constant_size: 128,
///         ..Default::default()
///     }).build();
///
///
/// gpu.encode(|gpu, encoder|{
/// });
/// ```
///
///
pub struct GPUContextBuilder<'gcb>{
    request_adapter_options: wgpu::RequestAdapterOptions<'gcb>,
    device_descriptor: wgpu::DeviceDescriptor<'gcb>,
    pub(crate) backends: wgpu::Backends,
}

impl<'gcb> Default for GPUContextBuilder<'gcb>{
    fn default() -> Self {
        let request_adapter_options = wgpu::RequestAdapterOptions{
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        };
        let  device_descriptor = wgpu::DeviceDescriptor{
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        };
        let backends = wgpu::Backends::all();
        Self{
            request_adapter_options,
            device_descriptor,
            backends,
        }
    }
}

impl<'gcb> GPUContextBuilder<'gcb>{
    pub fn new() -> Self{
        Self::default()
    }

    pub fn set_device_descriptor(mut self, device_descriptor: wgpu::DeviceDescriptor<'gcb>) -> Self{
        self.device_descriptor = device_descriptor;
        self
    }

    pub fn set_request_adapter_options(mut self, request_adapter_options: wgpu::RequestAdapterOptions<'gcb>) -> Self{
        self.request_adapter_options = request_adapter_options;
        self
    }

    pub fn set_power_preference(mut self, power_preference: wgpu::PowerPreference) -> Self{
        self.request_adapter_options.power_preference = power_preference;
        self
    }

    pub fn set_compatible_surface(mut self, surface: Option<&'gcb wgpu::Surface>) -> Self{
        self.request_adapter_options.compatible_surface = surface;
        self
    }

    pub fn set_force_fallback_addapter(mut self, force_fallback_adapter: bool) -> Self{
        self.request_adapter_options.force_fallback_adapter = force_fallback_adapter;
        self
    }

    pub fn set_features(mut self, features: wgpu::Features) -> Self{
        self.device_descriptor.features = features;
        self
    }

    pub fn enable_feature(mut self, features: wgpu::Features) -> Self{
        self.device_descriptor.features |= features;
        self
    }

    pub fn disable_feature(mut self, features: wgpu::Features) -> Self{
        self.device_descriptor.features &= !features;
        self
    }

    pub fn set_features_util(self) -> Self{
        self.enable_feature(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
            .enable_feature(wgpu::Features::VERTEX_WRITABLE_STORAGE)
            .enable_feature(wgpu::Features::PUSH_CONSTANTS)
            .enable_feature(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
            .enable_feature(wgpu::Features::POLYGON_MODE_LINE)
    }

    pub fn set_features_default(mut self) -> Self{
        self.device_descriptor.features = wgpu::Features::default();
        self
    }

    pub fn set_features_all_native(mut self) -> Self{
        self.device_descriptor.features = wgpu::Features::all_native_mask();
        self
    }

    pub fn set_features_all_webgpu(mut self) -> Self{
        self.device_descriptor.features = wgpu::Features::all_webgpu_mask();
        self
    }

    pub fn set_limits(mut self, limits: wgpu::Limits) -> Self{
        self.device_descriptor.limits = limits;
        self
    }

    pub fn set_device_label(mut self, label: wgpu::Label<'gcb>) -> Self{
        self.device_descriptor.label = label;
        self
    }

    pub fn build_with_instance(&self, instance: wgpu::Instance) -> GPUContext{
        pollster::block_on(self.build_with_instance_async(instance))
    }

    pub async fn build_with_instance_async(&self, instance: wgpu::Instance) -> GPUContext{
        let adapter = instance.request_adapter(
            &self.request_adapter_options
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &self.device_descriptor,
            None,
        ).await.unwrap();

        GPUContext{
            device,
            queue,
            adapter,
            instance,
            time: Instant::now(),
            dt: Duration::from_secs(1),
        }
    } 

    pub fn build(&self) -> GPUContext{
        pollster::block_on(self.build_async())
    }

    pub async fn build_async(&self) -> GPUContext{

        let instance = wgpu::Instance::new(self.backends);
        /*
        let instance = if let Some(instance) = self.instance{
            instance
        } else{
            wgpu::Instance::new(self.backends)
        };
        */


        let adapter = instance.request_adapter(
            &self.request_adapter_options
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &self.device_descriptor,
            None,
        ).await.unwrap();

        GPUContext{
            device,
            queue,
            adapter,
            instance,
            time: Instant::now(),
            dt: Duration::from_secs(1),
        }
    }

}

pub struct GPUContext{
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    pub time: Instant,
    pub dt: Duration,
}

impl GPUContext{
    pub fn new(instance: wgpu::Instance, surface: Option<&wgpu::Surface>) -> Self{
        pollster::block_on(Self::new_async(instance, surface))
    }

    pub async fn new_async(instance: wgpu::Instance, surface: Option<&wgpu::Surface>) -> Self{
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
                    .union(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
                    .union(wgpu::Features::POLYGON_MODE_LINE),
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
            instance,
            time: Instant::now(),
            dt: Duration::from_secs(1),
        }
    }
    pub(crate) fn update(&mut self) {
        let time = Instant::now();
        self.dt = time - self.time;
        self.time = time;
    }
    pub fn encode<F>(&mut self, mut f: F)
        where F: FnMut(&mut GPUContext, &mut wgpu::CommandEncoder){

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

            f(self, &mut encoder);

            self.queue.submit(Some(encoder.finish()));
    }
    pub fn encode_img<F>(&mut self, size: [u32; 2], mut f: F) -> image::DynamicImage
        where F: FnMut(&mut GPUContext, &wgpu::TextureView, &mut wgpu::CommandEncoder)
    {
        let o_tex = TextureBuilder::new()
            .clear(size)
            .format(wgpu::TextureFormat::Rgba8Unorm)
            .build(&self.device, &self.queue);

        let o_tex_view = o_tex.view_default();

        self.encode(|gpu, encoder|{
            f(gpu, &o_tex_view, encoder);
        });
        o_tex.slice(.., .., ..).to_image(&self.device)
    }
}
