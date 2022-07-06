use crate::{
    graphics::{Graphics, Quad, Renderable},
    map::{MeshOffsets, MAP_SIZE}, input::InputManager,
};

pub struct Snake {
    head: Head,
    body: Vec<Body>,
    queued_direction: Option<Direction>,

    vertex_buffer: wgpu::Buffer,
    vertices: u32,
}

impl Snake {
    pub fn new(gfx: &Graphics) -> Self {
        let vertex_buffer = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Snake Vertex Buffer"),
            size: (std::mem::size_of::<Quad>() * (MAP_SIZE * MAP_SIZE))
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let body = vec![Body { pos: (6, 5).into() }, Body { pos: (7, 5).into() }, Body { pos: (8, 5).into() }];

        Self {
            head: Head::default(),
            body,
            queued_direction: None,

            vertex_buffer,
            vertices: 0,
        }
    }

    pub fn update(&mut self, gfx: &Graphics, mesh_offsets: MeshOffsets) {
        if let Some(dir) = self.queued_direction {
            self.head.change_dir(dir);
        }
        self.queued_direction = None;
        let mut last_pos = self.head.pos;
        self.head.advance();
        
        for b in &mut self.body {
            let current_pos = b.pos;
            b.advance_to(last_pos.clone());
            last_pos = current_pos;
        }

        self.update_mesh(gfx, mesh_offsets);
    }

    fn update_mesh(&mut self, gfx: &Graphics, mesh_offsets: MeshOffsets) {
        let vertices = self.generate_vertices(mesh_offsets);
        self.vertices = vertices.len() as u32;

        gfx.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
    }

    fn generate_vertices(&self, mesh_offsets: MeshOffsets) -> Vec<Quad> {
        let mut vertices = Vec::new();

        let global_z = 0.0;
        let x_offset = mesh_offsets.x_offset;
        let y_offset = mesh_offsets.y_offset;
        let pos_to_coords_factor_x = mesh_offsets.map_width / MAP_SIZE as f32;
        let pos_to_coords_factor_y = mesh_offsets.map_height / MAP_SIZE as f32;
        let shorten_by_factor = 0.05;
        let shorten_width_by = shorten_by_factor * pos_to_coords_factor_x;
        let shorten_height_by = shorten_by_factor * pos_to_coords_factor_y;

        let temp_tex = [1.0, 1.0];  // <--- TODO change

        // Head
        let (pos_x, pos_y) = (
            mesh_offsets.left_x
                + pos_to_coords_factor_x * self.head.pos.x_tile as f32,
            mesh_offsets.bottom_y
                + pos_to_coords_factor_y * self.head.pos.y_tile as f32,
        );
        let top_left = [
            pos_x + shorten_width_by,
            pos_y - shorten_height_by,
            global_z,
        ];
        let bottom_right = [
            pos_x + x_offset - shorten_width_by,
            pos_y - y_offset + shorten_height_by,
        ];
        let head = Quad {
            top_left,
            bottom_right,
            tex_top_left: temp_tex,
            tex_bottom_right: temp_tex,
        };
        vertices.push(head);

        for b in &self.body {
            let (pos_x, pos_y) = (
                mesh_offsets.left_x
                    + pos_to_coords_factor_x * b.pos.x_tile as f32,
                mesh_offsets.bottom_y
                    + pos_to_coords_factor_y * b.pos.y_tile as f32,
            );
            let top_left = [
                pos_x + shorten_width_by,
                pos_y - shorten_height_by,
                global_z,
            ];
            let bottom_right = [
                pos_x + x_offset - shorten_width_by,
                pos_y - y_offset + shorten_height_by,
            ];
            let head = Quad {
                top_left,
                bottom_right,
                tex_top_left: temp_tex,
                tex_bottom_right: temp_tex,
            };
            vertices.push(head);
        }

        vertices
    }

    pub fn process_input(&mut self, input: &InputManager) {
        if let Some(key) = input.get_keyboard_state() {
            if key.is_pressed() {
                self.queued_direction = Some(match key.keycode {
                    winit::event::VirtualKeyCode::W => Direction::Up,
                    winit::event::VirtualKeyCode::A => Direction::Left,
                    winit::event::VirtualKeyCode::S => Direction::Down,
                    winit::event::VirtualKeyCode::D => Direction::Right,
                    _ => return
                })
            }
        }
    }
}

impl Renderable for Snake {
    fn render<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        gfx: &'a Graphics,
    ) {
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        rpass.draw(0..4, 0..self.vertices);
    }
}

#[derive(Debug)]
struct Head {
    pos: Position,
    direction: Direction,
}

impl Head {
    fn advance(&mut self) {
        match self.direction {
            Direction::Down => match self.pos.try_decrease_y() {
                Ok(_) => (),
                Err(_) => {
                    self.pos = (self.pos.x_tile, MAP_SIZE as u32).into()
                }
            },
            Direction::Up => {
                match self.pos.increase_y_with_bound_check(MAP_SIZE as u32)
                {
                    Ok(_) => (),
                    Err(_) => self.pos = (self.pos.x_tile, 1).into(),
                }
            }
            Direction::Left => match self.pos.try_decrease_x() {
                Ok(_) => (),
                Err(_) => {
                    self.pos = (MAP_SIZE as u32 - 1, self.pos.y_tile).into()
                }
            },
            Direction::Right => {
                match self.pos.increase_x_with_bound_check(MAP_SIZE as u32 - 1)
                {
                    Ok(_) => (),
                    Err(_) => self.pos = (0, self.pos.y_tile).into(),
                }
            }
        }
    }

    fn change_dir(&mut self, new_dir: Direction) {
        if new_dir == Direction::Up && self.direction != Direction::Down {
            self.direction = new_dir;
        } else if new_dir == Direction::Left && self.direction != Direction::Right {
            self.direction = new_dir;
        } else if new_dir == Direction::Down && self.direction != Direction::Up {
            self.direction = new_dir;
        } else if new_dir == Direction::Right && self.direction != Direction::Left {
            self.direction = new_dir;
        }
    }
}

impl Default for Head {
    fn default() -> Self {
        Self {
            pos: (5, 5).into(),
            direction: Direction::Up,
        }
    }
}

#[derive(Debug)]
struct Body {
    pos: Position,
}

impl Body {
    fn new(x_tile: u32, y_tile: u32) -> Self {
        Self {
            pos: (x_tile, y_tile).into(),
        }
    }

    fn advance_to(&mut self, pos: Position) {
        self.pos = pos;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x_tile: u32,
    y_tile: u32,
}

impl Position {
    fn increase_x(&mut self) {
        self.x_tile += 1;
    }
    fn increase_y(&mut self) {
        self.y_tile += 1;
    }

    fn try_decrease_x(&mut self) -> Result<(), ()> {
        self.x_tile = self.x_tile.checked_sub(1).ok_or(())?;
        Ok(())
    }

    fn try_decrease_y(&mut self) -> Result<(), ()> {
        self.y_tile = self.y_tile.checked_sub(1).ok_or(())?;
        Ok(())
    }

    fn increase_x_with_bound_check(&mut self, bound: u32) -> Result<(), ()> {
        if self.x_tile + 1 > bound {
            return Err(());
        }
        self.increase_x();

        Ok(())
    }

    fn increase_y_with_bound_check(&mut self, bound: u32) -> Result<(), ()> {
        if self.y_tile + 1 > bound {
            return Err(());
        }
        self.increase_y();

        Ok(())
    }
}

impl From<(u32, u32)> for Position {
    fn from(tile: (u32, u32)) -> Self {
        Self {
            x_tile: tile.0,
            y_tile: tile.1,
        }
    }
}
