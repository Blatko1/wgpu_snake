use crate::{
    graphics::Renderable, input::InputManager, world::World, Graphics,
};

pub struct Engine {
    pub input: InputManager,
    world: World,
}

impl Engine {
    pub fn new(gfx: &Graphics) -> Self {
        let input = InputManager::init();
        let world = World::new(gfx);
        Self { input, world }
    }

    pub fn render(&self, gfx: &Graphics) -> Result<(), wgpu::SurfaceError> {
        let backend = &gfx.backend;
        let (frame, view) = gfx.prepare_next_frame()?;
        let view = &view;

        let mut encoder = backend.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Main Command Encoder"),
            },
        );

        //Main Render Pass, render all main objects (snake, map, ...)
        {
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.627451,
                                    g: 0.909804,
                                    b: 0.125490,
                                    a: 1.,
                                }),
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            self.world.render(&mut rpass, &gfx);
        }

        backend.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn update(&mut self) {
        self.process_input();
    }

    fn process_input(&mut self) {
        let input = &mut self.input;

        input.reset();
    }

    pub fn on_resize(&mut self, gfx: &Graphics) {
        self.world.on_resize(gfx);
    }
}
