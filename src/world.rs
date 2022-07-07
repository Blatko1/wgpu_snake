use crate::{
    graphics::{Graphics, Quad, Renderable},
    map::Map,
    snake::Snake, input::InputManager,
};

pub struct World {
    snake: Snake,
    map: Map,

    pipeline: wgpu::RenderPipeline,
}

impl World {
    pub fn new(gfx: &Graphics) -> Self {
        let snake = Snake::new(gfx);
        let map = Map::new(gfx);

        let shader_module = gfx
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders\\world.wgsl"));

        let layout = gfx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("World Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            },
        );

        let pipeline = gfx.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("World Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[Quad::vertex_buffer_layout()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gfx.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            },
        );

        Self {
            snake,
            map,
            pipeline,
        }
    }

    pub fn update(&mut self, gfx: &Graphics) {
        self.snake.update(gfx, self.map.offsets);
    }

    pub fn on_resize(&mut self, gfx: &Graphics) {
        self.map.resize_map(gfx);
    }

    pub fn process_input(&mut self, input: &InputManager) {
        self.snake.process_input(input);
    }
}

// Map, player, UI, Text, items are all rendered in different draw calls

impl Renderable for World {
    fn render<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>
    ) {
        self.map.render(rpass);

        rpass.set_pipeline(&self.pipeline);

        self.snake.render(rpass);
    }
}
