use crate::{
    game_elements::{AppleGen, Snake, Position},
    graphics::{Graphics, Quad, Renderable},
    input::InputManager,
    map::Map,
};

pub const MAP_SIZE: usize = 15;
pub const STARTING_POS: Position = Position {
    x_tile: 4,
    y_tile: 4,
};
pub const STARTING_SNAKE_SIZE: usize = 3;
pub const APPLE_WORTH: usize = 5;

pub struct Game {
    snake: Snake,
    apple: AppleGen,
    map: Map,

    pipeline: wgpu::RenderPipeline,
    /*bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout*/
}

impl Game {
    pub fn new(gfx: &Graphics) -> Self {
        let map = Map::new(gfx);
        let snake = Snake::new(gfx);
        let apple = AppleGen::new(gfx);

        apple.update_mesh(gfx, map.offsets);

        let shader_module = gfx
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/quad.wgsl"));

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



        /*let size = wgpu::Extent3d {
            width: todo!(),
            height: todo!(),
            depth_or_array_layers: 1,
        };

        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let bind_group = gfx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: (), view_dimension: (), multisampled: () },
                    count: todo!(),
                }
            ],
        })*/

        Self {
            snake,
            apple,
            map,
            pipeline,
        }
    }

    pub fn update(&mut self, gfx: &Graphics) {
        self.snake.update(gfx, &mut self.apple, &mut self.map);
        self.map.update_tiles_data(self.snake.update_tile_data());
        self.apple.update(gfx, &self.map);
        //println!("{}", self.map);
    }

    pub fn on_resize(&mut self, gfx: &Graphics) {
        self.map.resize_map(gfx);
    }

    pub fn process_input(&mut self, input: &InputManager) {
        self.snake.process_input(input);
    }
}

// Map, player, UI, Text, items are all rendered in different draw calls.

impl Renderable for Game {
    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        self.map.render(rpass);

        rpass.set_pipeline(&self.pipeline);

        self.snake.render(rpass);
        self.apple.render(rpass);
    }
}
