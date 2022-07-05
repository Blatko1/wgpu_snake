use wgpu::util::DeviceExt;

use crate::{graphics::{Renderable, Backend, Graphics, Vertex, QUAD}, map::MAP_SIZE};

pub struct Snake {
    head: Head,
    body: Vec<Body>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices: u32
}

impl Snake {
    pub fn new(gfx: &Graphics) -> Self {
        let backend = &gfx.backend;
    
        let vertex_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Snake Vertex Buffer"),
            size: (std::mem::size_of::<Vertex>() * 4 * (MAP_SIZE * MAP_SIZE)) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indices_size = std::mem::size_of::<u16>() * 6 * (MAP_SIZE * MAP_SIZE);

        let index_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Snake Index Buffer"),
            size: indices_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { head: Head::default(), body: Vec::new(), vertex_buffer, index_buffer, indices: indices_size as u32 }
    }

    pub fn update(&mut self) {
        self.head.advance();
    }

    fn update_mesh(&self, backend: &Backend) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        
    }
}

impl Renderable for Snake {
    fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>, gfx: &'a crate::graphics::Graphics) {
        
    }
}

#[derive(Debug)]
struct Head {
    pos: Position,
    direction: Direction
}

impl Head {
    fn advance(&mut self) {
        match self.direction {
            Direction::Up => match self.pos.try_decrease_y() {
                Ok(_) => (),
                Err(_) => self.pos = (self.pos.x_tile, MAP_SIZE as u32 - 1).into(),
            },
            Direction::Down => match self.pos.increase_y_with_bound_check(MAP_SIZE as u32 - 1) {
                Ok(_) => (),
                Err(_) => self.pos = (self.pos.x_tile, 0).into(),
            },
            Direction::Left => match self.pos.try_decrease_x() {
                Ok(_) => (),
                Err(_) => self.pos = (MAP_SIZE as u32 - 1, self.pos.y_tile).into(),
            },
            Direction::Right => match self.pos.increase_x_with_bound_check(MAP_SIZE as u32 - 1) {
                Ok(_) => (),
                Err(_) => self.pos = (0, self.pos.y_tile).into(),
            },
        }
    }
}

impl Default for Head {
    fn default() -> Self {
        Self { pos: (20, 15).into(), direction: Direction::Up }
    }
}

#[derive(Debug)]
struct Body {
    pos: Position
}

impl Body {
    fn new(x_tile: u32, y_tile: u32) -> Self {
        Self { pos: (x_tile, y_tile).into() }
    }

    fn advance(&self) {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up, Down, Left, Right
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x_tile: u32,
    y_tile: u32
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