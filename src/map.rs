use wgpu::util::DeviceExt;

use crate::graphics::{LineVertex, Renderable, Graphics};

pub const MAP_SIZE: usize = 30;

pub struct Map {
    fields: [bool; MAP_SIZE],

    pipeline: wgpu::RenderPipeline,
    mesh: MapMesh,
}

impl Map {
    pub fn new(gfx: &Graphics) -> Self {
        let backend = &gfx.backend;
        let fields = [false; MAP_SIZE];

        let shader_module = backend
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders\\map.wgsl"));

        let layout = backend.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("World Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            },
        );

        let pipeline = backend.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("World Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[LineVertex::vertex_buffer_layout()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
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
                        format: backend.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            },
        );

        let mesh = Self::_new_map_mesh(gfx);

        Self {
            fields,
            pipeline,
            mesh,
        }
    }

    fn _new_map_mesh(gfx: &Graphics) -> MapMesh {
        let mut vertices = Vec::new();

        let backend = &gfx.backend;
        let win_width = backend.config.width;
        let win_height = backend.config.height;

        // Calculate bounds of the map in coordinates.
        let (left_x, right_x, top_y, bottom_y, x_line_offset, y_line_offset) = match win_width.cmp(&win_height) {
            std::cmp::Ordering::Less => {
                let half_of_uncovered_height = (win_height - win_width) as f32 / 2.0;
                let percentage_of_the_half = half_of_uncovered_height / win_height as f32;
                let delta_of_uncovered_half_in_coords = percentage_of_the_half * 2.0;
                let top = 1.0 - delta_of_uncovered_half_in_coords;
                let bottom = -1.0 + delta_of_uncovered_half_in_coords;
                let left: f32 = -1.0;
                let right: f32 = 1.0;
                let x_line_offset = 2.0 / MAP_SIZE as f32;
                let y_line_offset = (top + bottom.abs()) / MAP_SIZE as f32;
                (left, right, top, bottom, x_line_offset, y_line_offset)
            },
            std::cmp::Ordering::Equal => (-1.0, 1.0, 1.0, -1.0, 2.0 / MAP_SIZE as f32, 2.0 / MAP_SIZE as f32),
            std::cmp::Ordering::Greater => {
                let half_of_uncovered_width = (win_width - win_height) as f32 / 2.0;
                let percentage_of_the_half = half_of_uncovered_width / win_width as f32;
                let delta_of_uncovered_half_in_coords = percentage_of_the_half * 2.0;
                let top = 1.0;
                let bottom = -1.0;
                let left: f32 = -1.0 + delta_of_uncovered_half_in_coords;
                let right: f32 = 1.0 - delta_of_uncovered_half_in_coords;
                let x_line_offset = (right + left.abs()) / MAP_SIZE as f32;
                let y_line_offset = 2.0 / MAP_SIZE as f32;
                (left, right, top, bottom, x_line_offset, y_line_offset)
            },
        };

        // Vertical lines
        for l in 0..MAP_SIZE {
            let offset_x = l as f32 * x_line_offset;
            vertices.push(line(left_x + offset_x, top_y));
            vertices.push(line(left_x + offset_x, bottom_y));
        }

        // Last vertical line
        vertices.push(line(right_x, top_y));
        vertices.push(line(right_x, bottom_y));

        // Horizontal lines
        for l in 0..MAP_SIZE {
            let offset_y = l as f32 * y_line_offset;
            vertices.push(line(left_x, bottom_y + offset_y));
            vertices.push(line(right_x, bottom_y + offset_y));
        }

        // Last horizontal line
        vertices.push(line(left_x, top_y));
        vertices.push(line(right_x, top_y));

        let vertex_buffer =
        backend.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Map Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        MapMesh {
            vertex_buffer,
            vertices_count: vertices.len() as u32,
        }
    }

    pub fn resize_map(&mut self, gfx: &Graphics) {
        self.mesh = Self::_new_map_mesh(gfx);
    }
}

fn line(x: f32, y: f32) -> LineVertex {
    LineVertex { pos: [x, y] }
}

impl Renderable for Map {
    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>, gfx: &'a Graphics) {
        rpass.set_pipeline(&self.pipeline);

        rpass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));

        rpass.draw(0..self.mesh.vertices_count, 0..1);
    }
}

struct MapMesh {
    vertex_buffer: wgpu::Buffer,
    vertices_count: u32,
}
