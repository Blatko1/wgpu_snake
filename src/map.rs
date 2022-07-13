use std::fmt::{Display, Write};

use wgpu::util::DeviceExt;

use crate::{
    game_elements::{Position, TileUpdateData},
    graphics::{Graphics, LineVertex, Quad, Renderable}, game::MAP_SIZE,
};

pub struct Map {
    // Tile array marks occupied and unoccupied tiles.
    pub tiles: [Tile; MAP_SIZE * MAP_SIZE],
    pub offsets: MeshOffsets,

    pipeline: wgpu::RenderPipeline,
    mesh: MapMesh,
}

impl Map {
    pub fn new(gfx: &Graphics) -> Self {
        let tiles = Self::generate_tiles();

        let shader_module = gfx
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/line.wgsl"));

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
                        format: gfx.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            },
        );

        let (mesh, offsets) = Self::_new_map_mesh(gfx);

        Self {
            tiles,
            offsets,
            pipeline,
            mesh,
        }
    }

    pub fn update_tiles_data(&mut self, data: TileUpdateData) {
        let occupy_tile_index = Self::pos_to_tile_index(data.occupy);
        self.tiles[occupy_tile_index].is_occupied = true;
        if let Some(unoccupy) = data.unoccupy {
            let unoccupy_tile_index = Self::pos_to_tile_index(unoccupy);
            self.tiles[unoccupy_tile_index].is_occupied = false;
        }
    }

    fn _new_map_mesh(gfx: &Graphics) -> (MapMesh, MeshOffsets) {
        let mut vertices = Vec::new();

        let win_width = gfx.config.width;
        let win_height = gfx.config.height;

        // Calculate bounds of the map in coordinates.
        let (left_x, right_x, top_y, bottom_y, x_line_offset, y_line_offset) =
            match win_width.cmp(&win_height) {
                std::cmp::Ordering::Less => {
                    let half_of_uncovered_height =
                        (win_height - win_width) as f32 / 2.0;
                    let percentage_of_the_half =
                        half_of_uncovered_height / win_height as f32;
                    let delta_of_uncovered_half_in_coords =
                        percentage_of_the_half * 2.0;
                    let top = 1.0 - delta_of_uncovered_half_in_coords;
                    let bottom = -1.0 + delta_of_uncovered_half_in_coords;
                    let left: f32 = -1.0;
                    let right: f32 = 1.0;
                    let x_line_offset = 2.0 / MAP_SIZE as f32;
                    let y_line_offset = (top + bottom.abs()) / MAP_SIZE as f32;
                    (left, right, top, bottom, x_line_offset, y_line_offset)
                }
                std::cmp::Ordering::Equal => (
                    -1.0,
                    1.0,
                    1.0,
                    -1.0,
                    2.0 / MAP_SIZE as f32,
                    2.0 / MAP_SIZE as f32,
                ),
                std::cmp::Ordering::Greater => {
                    let half_of_uncovered_width =
                        (win_width - win_height) as f32 / 2.0;
                    let percentage_of_the_half =
                        half_of_uncovered_width / win_width as f32;
                    let delta_of_uncovered_half_in_coords =
                        percentage_of_the_half * 2.0;
                    let top = 1.0;
                    let bottom = -1.0;
                    let left: f32 = -1.0 + delta_of_uncovered_half_in_coords;
                    let right: f32 = 1.0 - delta_of_uncovered_half_in_coords;
                    let x_line_offset = (right + left.abs()) / MAP_SIZE as f32;
                    let y_line_offset = 2.0 / MAP_SIZE as f32;
                    (left, right, top, bottom, x_line_offset, y_line_offset)
                }
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
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Map Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let offsets = MeshOffsets {
            left_x,
            bottom_y,
            x_offset: x_line_offset,
            y_offset: y_line_offset,
            map_width: left_x.abs() + right_x,
            map_height: top_y + bottom_y.abs(),
        };

        (
            MapMesh {
                vertex_buffer,
                vertices_count: vertices.len() as u32,
            },
            offsets,
        )
    }

    pub fn resize_map(&mut self, gfx: &Graphics) {
        let (mesh, offsets) = Self::_new_map_mesh(gfx);
        self.mesh = mesh;
        self.offsets = offsets;
    }

    pub fn is_tile_occupied(&self, pos: Position) -> bool {
        let index = Self::pos_to_tile_index(pos);
        self.tiles[index].is_occupied
    }

    pub fn reset(&mut self) {
        self.tiles = Self::generate_tiles();
    }

    fn pos_to_tile_index<P: Into<Position>>(pos: P) -> usize {
        let pos = pos.into();
        (pos.y_tile * MAP_SIZE as u32 + pos.x_tile - MAP_SIZE as u32) as usize
    }

    fn generate_tiles() -> [Tile; MAP_SIZE * MAP_SIZE] {
        let mut tiles = Vec::new();
        for i in 1..=MAP_SIZE {
            for j in 0..MAP_SIZE {
                tiles.push(Tile {
                    is_occupied: false,
                    pos: (j as u32, i as u32).into(),
                })
            }
        }
        tiles.try_into().unwrap()
    }
}

fn line(x: f32, y: f32) -> LineVertex {
    LineVertex { pos: [x, y] }
}

impl Renderable for Map {
    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_pipeline(&self.pipeline);

        rpass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));

        rpass.draw(0..self.mesh.vertices_count, 0..1);
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tile_width = 3;
        let width = MAP_SIZE * tile_width + 1;
        for i in 1..=MAP_SIZE {
            f.write_str(&format!(
                "\n{blank:->width$}\n",
                blank = '-',
                width = width
            ))?;
            f.write_char('|')?;
            for j in 0..MAP_SIZE {
                let tile = &self.tiles[Self::pos_to_tile_index((
                    j as u32,
                    MAP_SIZE as u32 + 1 - i as u32,
                ))];
                if tile.is_occupied {
                    f.write_char('✅')?;
                } else {
                    f.write_char('❌')?;
                }
                f.write_char('|')?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Tile {
    pub is_occupied: bool,
    pub pos: Position,
}

struct MapMesh {
    vertex_buffer: wgpu::Buffer,
    vertices_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct MeshOffsets {
    pub left_x: f32,
    pub bottom_y: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub map_width: f32,
    pub map_height: f32,
}

impl MeshOffsets {
    pub fn calculate_factors(&self) -> MeshFactors {
        // Factors used for turning map position (0-30) to real coordinates.
        let to_x_coord_factor = self.map_width / MAP_SIZE as f32;
        let to_y_coord_factor = self.map_height / MAP_SIZE as f32;
        let shorten_by_factor = 0.05;
        let shorten_width = shorten_by_factor * to_x_coord_factor;
        let shorten_height = shorten_by_factor * to_y_coord_factor;

        MeshFactors {
            to_x_coord_factor,
            to_y_coord_factor,
            shorten_width,
            shorten_height,
        }
    }
}

pub struct MeshFactors {
    pub to_x_coord_factor: f32,
    pub to_y_coord_factor: f32,
    pub shorten_width: f32,
    pub shorten_height: f32,
}

pub trait ElementMesh {
    fn generate_mesh(&self, offsets: MeshOffsets) -> Vec<Quad>;
}
