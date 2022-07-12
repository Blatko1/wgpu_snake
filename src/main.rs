mod engine;
mod game_elements;
mod graphics;
mod input;
mod map;
mod game;

use std::time::{Duration, Instant};

use engine::Engine;
use graphics::Graphics;
use pollster::block_on;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("cycles")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 400))
        .build(&event_loop)
        .unwrap();

    let mut graphics = block_on(Graphics::new(&window));
    let mut engine = Engine::new(&graphics);

    let framerate_delta = Duration::from_secs_f64(1.0 / 60.0);
    let mut time_delta = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => engine.input.keyboard_input(state, keycode),
                winit::event::WindowEvent::Resized(new_size)
                | winit::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut new_size,
                    ..
                } => {
                    graphics.resize(new_size);
                    engine.on_resize(&graphics)
                }
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                engine.update(&graphics);

                match engine.render(&graphics) {
                    Ok(_) => (),
                    Err(wgpu::SurfaceError::Lost) => {
                        println!(
                            "__________SWAP CHAIN HAS BEEN LOST\
                         AND NEEDS TO BE RECREATED!!!____________"
                        );
                        println!(
                            "__________RECREATING SWAP CHAIN!!!____________"
                        );
                        graphics.resize(graphics.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        println!("__________OUT OF MEMORY!!!____________");
                        *control_flow = ControlFlow::Exit
                    }
                    Err(e) => eprintln!("ERROR: {}", e),
                }
            }
            winit::event::Event::MainEventsCleared => {
                // Regulate to 60 FPS
                let elapsed = time_delta.elapsed();

                if framerate_delta <= elapsed {
                    window.request_redraw();
                    time_delta = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + framerate_delta - elapsed,
                    );
                }
            }
            winit::event::Event::DeviceEvent {
                event: winit::event::DeviceEvent::MouseMotion { delta },
                ..
            } => engine.input.mouse_input(delta),
            _ => (),
        }
    });
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    run();
}
