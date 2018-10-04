use glium::glutin::{ElementState, Event, GlWindow, MouseButton, MouseScrollDelta, WindowEvent};
use imgui::{ImGui, Ui};
use renderer::camera::PCamera;
use renderer::Vec2;

pub fn handle_input(event: &Event, camera: &mut PCamera) {
    match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(ref dims) => camera.set_aspect_from_dims((*dims).into()),
            _ => (),
        },
        _ => (),
    }
}

pub fn get_resized(event: &Event) -> Option<(f64, f64)> {
    match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(value) => Some((*value).into()),
            _ => None,
        },
        _ => None,
    }
}

pub struct Mouse {
    pub position: Vec2,
    pub movement: Vec2,
    pub pressed: (bool, bool, bool),
    pub wheel: f32,
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            position: Vec2::new(0.0, 0.0),
            movement: Vec2::new(0.0, 0.0),
            pressed: (false, false, false),
            wheel: 0.0,
        }
    }

    pub fn move_to(&mut self, pos: Vec2) {
        self.movement = pos - self.position;
        self.position = pos;
    }

    pub fn move_to_tuple(&mut self, pos: (f32, f32)) {
        let pos = Vec2::new(pos.0, pos.1);
        self.move_to(pos);
    }
}

pub struct MouseState {
    pub mouse: Mouse,
    pub pressed_on_ui: bool,
    pub on_ui: bool,
}

impl MouseState {
    pub fn new() -> MouseState {
        MouseState {
            mouse: Mouse::new(),
            pressed_on_ui: false,
            on_ui: false,
        }
    }

    pub fn update_mouse(&self, ui: &mut ImGui) {
        ui.set_mouse_down([
            self.mouse.pressed.0,
            self.mouse.pressed.1,
            self.mouse.pressed.2,
            false,
            false,
        ]);
        ui.set_mouse_wheel(self.mouse.wheel);
        ui.set_mouse_pos(self.mouse.position.x.round(), self.mouse.position.y.round());
    }

    pub fn update_on_ui(&mut self, ui: &Ui) {
        self.on_ui = ui.want_capture_mouse();
    }

    pub fn mouse_one_down(&mut self, state: bool, ui: &Ui) {
        self.pressed_on_ui = ui.want_capture_mouse() && state;
        self.mouse.pressed.0 = state;
    }

    pub fn get_drag_off_ui(&self) -> Option<Vec2> {
        if !self.pressed_on_ui && self.mouse.pressed.0 {
            return Some(self.mouse.movement);
        }
        None
    }

    pub fn reset(&mut self) {
        self.mouse.movement = Vec2::new(0.0, 0.0);
        self.mouse.wheel = 0.0;
    }

    pub fn handle_window_event(
        &mut self,
        event: &Event,
        ui: &Ui,
        window: &GlWindow,
        hidpi_factor: f64,
    ) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    let pos = position
                        .to_physical(window.get_hidpi_factor())
                        .to_logical(hidpi_factor);
                    self.mouse.move_to_tuple((pos.x as f32, pos.y as f32));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let y = match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(delta) => delta.y as f32,
                    };
                    self.mouse.wheel = y
                }
                WindowEvent::MouseInput { button, state, .. } => match button {
                    MouseButton::Left => self.mouse_one_down(*state == ElementState::Pressed, ui),
                    MouseButton::Right => self.mouse.pressed.1 = *state == ElementState::Pressed,
                    MouseButton::Middle => self.mouse.pressed.2 = *state == ElementState::Pressed,
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
    }
}
