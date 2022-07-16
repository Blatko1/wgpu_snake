use rand::Rng;

use crate::{
    graphics::{Graphics, Quad, Renderable},
    input::InputManager,
    map::{ElementMesh, Map, MeshOffsets, Tile}, game::{MAP_SIZE, APPLE_WORTH, STARTING_POS, STARTING_SNAKE_SIZE},
};

pub struct Snake {
    head: Head,
    body: Vec<Body>,
    queued_direction: Option<Direction>,
    // Option because there is no unoccupying
    // until the snake is at it's full starting size.
    last_unoccupied: Option<Position>,
    alive_time: usize,

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

        let body = Self::body();

        Self {
            head: Head::default(),
            body,
            queued_direction: None,
            last_unoccupied: None,
            alive_time: 0,

            vertex_buffer,
            vertices: 0,
        }
    }

    pub fn update(&mut self, gfx: &Graphics, apple: &mut AppleGen, map: &mut Map) {
        let offsets = map.offsets;
        if let Some(dir) = self.queued_direction {
            self.head.change_dir(dir);
        }
        self.queued_direction = None;
        self.last_unoccupied = if self.alive_time >= self.body.len() {
            Some(self.body.last().unwrap().pos)
        } else {
            None
        };

        let mut last_pos = self.head.pos;
        self.head.advance();

        for b in &mut self.body {
            let current_pos = b.pos;
            b.advance_to(last_pos);
            last_pos = current_pos;
        }

        if self.head.pos == apple.pos {
            apple.eat();
            self.extend(APPLE_WORTH);
        } else if map.is_tile_occupied(self.head.pos) {
            apple.eat();
            map.reset();
            self.reset();
        }

        self.update_mesh(gfx, offsets);

        self.alive_time += 1;
    }

    fn extend(&mut self, add: usize) {
        let tail_pos = self.body.last().unwrap().pos;
        for _ in 0..add {
            self.body.push(Body {
                pos: tail_pos,
            });
        }
    }

    fn update_mesh(&mut self, gfx: &Graphics, offsets: MeshOffsets) {
        let vertices = self.generate_mesh(offsets);
        self.vertices = vertices.len() as u32;

        gfx.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
    }

    fn reset(&mut self) {
        self.head.pos = STARTING_POS;
        self.body = Self::body();
    }

    pub fn update_tile_data(&self) -> TileUpdateData {
        let occupy = self.head.pos;
        let unoccupy = self.last_unoccupied;

        TileUpdateData { occupy, unoccupy }
    }

    pub fn process_input(&mut self, input: &InputManager) {
        if self.queued_direction.is_some() {
            return;
        }
        if let Some(key) = input.get_keyboard_state() {
            use winit::event::VirtualKeyCode;
            if key.is_pressed() {
                self.queued_direction = Some(match key.keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => Direction::Up,
                    VirtualKeyCode::A | VirtualKeyCode::Left => Direction::Left,
                    VirtualKeyCode::S | VirtualKeyCode::Down => Direction::Down,
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        Direction::Right
                    }
                    _ => return,
                })
            }
        }
    }

    fn body() -> Vec<Body> {
        let mut body = Vec::new();
        for _ in 0..STARTING_SNAKE_SIZE {
            body.push(Body::default());
        }
        body
    }
}

impl ElementMesh for Snake {
    fn generate_mesh(&self, offsets: MeshOffsets) -> Vec<Quad> {
        let mut quads = Vec::new();

        let global_z = 0.0;
        let x_offset = offsets.x_offset;
        let y_offset = offsets.y_offset;
        let factors = offsets.calculate_factors();

        let tex_top_left = [0.8, 0.7];
        let tex_bottom_right = [0.9, 1.0];

        // Head
        let (pos_x, pos_y) = (
            offsets.left_x
                + self.head.pos.x_tile as f32 * factors.to_x_coord_factor,
            offsets.bottom_y
                + self.head.pos.y_tile as f32 * factors.to_y_coord_factor,
        );
        let top_left = [
            pos_x + factors.shorten_width,
            pos_y - factors.shorten_height,
            global_z,
        ];
        let bottom_right = [
            pos_x + x_offset - factors.shorten_width,
            pos_y - y_offset + factors.shorten_height,
        ];
        let head = Quad {
            top_left,
            bottom_right,
            tex_top_left,
            tex_bottom_right,
        };
        quads.push(head);

        let tex_top_left = [0.3, 0.1];
        let tex_bottom_right = [0.1, 0.9];

        for b in &self.body {
            let (pos_x, pos_y) = (
                offsets.left_x
                    + b.pos.x_tile as f32 * factors.to_x_coord_factor,
                offsets.bottom_y
                    + b.pos.y_tile as f32 * factors.to_y_coord_factor,
            );
            let top_left = [
                pos_x + factors.shorten_width,
                pos_y - factors.shorten_height,
                global_z,
            ];
            let bottom_right = [
                pos_x + x_offset - factors.shorten_width,
                pos_y - y_offset + factors.shorten_height,
            ];
            let head = Quad {
                top_left,
                bottom_right,
                tex_top_left,
                tex_bottom_right,
            };
            quads.push(head);
        }

        quads
    }
}

impl Renderable for Snake {
    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
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
            Direction::Down => match self.pos.decrease_y_with_bound(1) {
                Ok(_) => (),
                Err(_) => self.pos = (self.pos.x_tile, MAP_SIZE as u32).into(),
            },
            Direction::Up => {
                match self.pos.increase_y_with_bound(MAP_SIZE as u32) {
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
                match self.pos.increase_x_with_bound(MAP_SIZE as u32 - 1) {
                    Ok(_) => (),
                    Err(_) => self.pos = (0, self.pos.y_tile).into(),
                }
            }
        }
    }

    fn change_dir(&mut self, new_dir: Direction) {
        if new_dir == Direction::Up && self.direction != Direction::Down {
            self.direction = new_dir;
            return;
        }
        if new_dir == Direction::Left && self.direction != Direction::Right {
            self.direction = new_dir;
            return;
        }
        if new_dir == Direction::Down && self.direction != Direction::Up {
            self.direction = new_dir;
            return;
        }
        if new_dir == Direction::Right && self.direction != Direction::Left {
            self.direction = new_dir;
        }
    }
}

impl Default for Head {
    fn default() -> Self {
        Self {
            pos: STARTING_POS,
            direction: Direction::Up,
        }
    }
}

#[derive(Debug)]
struct Body {
    pos: Position,
}

impl Body {
    fn advance_to(&mut self, pos: Position) {
        self.pos = pos;
    }
}

impl Default for Body {
    fn default() -> Self {
        Self { pos: STARTING_POS }
    }
}

pub struct TileUpdateData {
    pub occupy: Position,
    pub unoccupy: Option<Position>,
}

pub struct AppleGen {
    pub pos: Position,
    is_eaten: bool,

    vertex_buffer: wgpu::Buffer,
}

impl AppleGen {
    pub fn new(gfx: &Graphics) -> AppleGen {
        let pos = (0, 0).into();

        let vertex_buffer = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Apple Vertex Buffer"),
            size: std::mem::size_of::<Quad>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        AppleGen {
            pos,
            is_eaten: true,
            vertex_buffer,
        }
    }

    fn find_unoccupied_pos(map: &Map) -> Position {
        let filtered: Vec<&Tile> =
            map.tiles.iter().filter(|x| !x.is_occupied).collect();
        let mut rand = rand::thread_rng();
        let index = rand.gen_range(0..filtered.len());
        filtered.get(index).unwrap().pos
    }

    pub fn update(&mut self, gfx: &Graphics, map: &Map) {
        if self.is_eaten {
            self.is_eaten = false;
            self.pos = Self::find_unoccupied_pos(map);
            self.update_mesh(gfx, map.offsets);
        }
    }

    pub fn update_mesh(&self, gfx: &Graphics, offsets: MeshOffsets) {
        let mesh = self.generate_mesh(offsets);
        gfx.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&mesh),
        );
    }

    pub fn eat(&mut self) {
        self.is_eaten = true;
    }
}

impl ElementMesh for AppleGen {
    fn generate_mesh(&self, offsets: MeshOffsets) -> Vec<Quad> {
        let mut quads = Vec::new();

        let global_z = 0.0;
        let x_offset = offsets.x_offset;
        let y_offset = offsets.y_offset;
        let factors = offsets.calculate_factors();

        let tex_top_left = [1.0, 1.0];
        let tex_bottom_right = [1.0, 0.0];

        let (pos_x, pos_y) = (
            offsets.left_x + self.pos.x_tile as f32 * factors.to_x_coord_factor,
            offsets.bottom_y
                + self.pos.y_tile as f32 * factors.to_y_coord_factor,
        );

        let top_left = [
            pos_x + factors.shorten_width,
            pos_y - factors.shorten_height,
            global_z,
        ];

        let bottom_right = [
            pos_x + x_offset - factors.shorten_width,
            pos_y - y_offset + factors.shorten_height,
        ];

        let apple = Quad {
            top_left,
            bottom_right,
            tex_top_left,
            tex_bottom_right,
        };

        quads.push(apple);

        quads
    }
}

impl Renderable for AppleGen {
    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        rpass.draw(0..4, 0..1);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x_tile: u32,
    pub y_tile: u32,
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

    fn increase_x_with_bound(&mut self, bound: u32) -> Result<(), ()> {
        if self.x_tile + 1 > bound {
            return Err(());
        }
        self.increase_x();

        Ok(())
    }

    fn decrease_y_with_bound(&mut self, bound: u32) -> Result<(), ()> {
        if self.y_tile - 1 < bound {
            return Err(());
        }
        self.try_decrease_y().unwrap();

        Ok(())
    }

    fn increase_y_with_bound(&mut self, bound: u32) -> Result<(), ()> {
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
