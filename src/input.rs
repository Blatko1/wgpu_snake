use winit::event::{ElementState, VirtualKeyCode};

#[derive(Debug)]
pub struct InputManager {
    keyboard_state: Option<KeyboardState>,
    mouse_delta: MouseDelta,
}

impl InputManager {
    pub fn init() -> Self {
        let keyboard_state = None;
        let mouse_delta = MouseDelta::default();
        println!("delta: {:?}", mouse_delta);
        Self {
            keyboard_state,
            mouse_delta,
        }
    }

    pub fn reset(&mut self) {
        self.keyboard_state = None;
        self.mouse_delta.delta_x = 0;
        self.mouse_delta.delta_y = 0;
    }

    pub fn get_keyboard_state(&self) -> Option<KeyboardState> {
        self.keyboard_state
    }

    pub fn get_mouse_delta(&self) -> MouseDelta {
        self.mouse_delta
    }

    pub fn keyboard_input(
        &mut self,
        state: ElementState,
        keycode: VirtualKeyCode,
    ) {
        self.keyboard_state = Some(KeyboardState { state, keycode })
    }

    pub fn mouse_input(&mut self, raw_delta: (f64, f64)) {
        self.mouse_delta.delta_x = raw_delta.0 as i32;
        self.mouse_delta.delta_y = -raw_delta.1 as i32;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyboardState {
    state: ElementState,
    keycode: VirtualKeyCode,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MouseDelta {
    delta_x: i32,
    delta_y: i32,
}
