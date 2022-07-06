#[repr(C)]
#[derive(bytemuck::Zeroable, bytemuck::Pod, Clone, Copy, Debug)]
pub struct LineVertex {
    pub pos: [f32; 2],
}

impl LineVertex {
    pub fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        }
    }
}

pub struct Graphics {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl Graphics {
    pub async fn new(window: &winit::window::Window) -> Self {
        let backends = wgpu::util::backend_bits_from_env()
            .unwrap_or_else(wgpu::Backends::all);
        let instance = wgpu::Instance::new(backends);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(window);
            (size, surface)
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Graphics Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let format = surface.get_supported_formats(&adapter)[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
            size,
        }
    }

    pub fn prepare_next_frame(
        &self,
    ) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView), wgpu::SurfaceError>
    {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(err) => return Err(err),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Ok((frame, view))
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
}

pub trait Renderable {
    fn render<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        gfx: &'a Graphics,
    );
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Quad {
    pub top_left: [f32; 3],
    pub bottom_right: [f32; 2],
    pub tex_top_left: [f32; 2],
    pub tex_bottom_right: [f32; 2],
}

impl Quad {
    pub fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Quad>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
            ],
        }
    }
}
