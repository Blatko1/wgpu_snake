use crate::{
    graphics::Renderable, input::InputManager, game::Game, Graphics,
};

pub struct Engine {
    pub input: InputManager,
    game: Game,

    tick_counter: u32,
}

impl Engine {
    pub fn new(gfx: &Graphics) -> Self {
        let input = InputManager::init();
        let game = Game::new(gfx);
        Self {
            input,
            game,
            tick_counter: 0,
        }
    }

    pub fn render(&self, gfx: &Graphics) -> Result<(), wgpu::SurfaceError> {
        let (frame, view) = gfx.prepare_next_frame()?;
        let view = &view;

        let mut encoder = gfx.device.create_command_encoder(
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

            self.game.render(&mut rpass);
        }

        gfx.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn update(&mut self, gfx: &Graphics) {
        self.process_input();

        self.tick_counter += 1;

        if self.tick_counter >= 10 {
            self.game.update(gfx);
            self.tick_counter = 0;
        }
    }

    fn process_input(&mut self) {
        let input = &mut self.input;

        self.game.process_input(input);

        input.reset();
    }

    pub fn on_resize(&mut self, gfx: &Graphics) {
        self.game.on_resize(gfx);
    }
}
