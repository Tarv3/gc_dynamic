use glium::backend::glutin::Display;
use glium::glutin::{
    dpi::LogicalSize, ContextBuilder, ElementState, Event, EventsLoop,
    VirtualKeyCode, WindowBuilder, WindowEvent,
};

pub struct Window {
    pub display: Display,
    pub aspect: f32,
    pub open: bool,
}
impl Window {
    pub fn new(
        fullscreen: bool,
        vsync: bool,
        visible: bool,
        screen_size: [f64; 2],
        events_loop: &EventsLoop,
    ) -> Window {
        let mut window: WindowBuilder = WindowBuilder::new().with_visibility(visible);
        if fullscreen {
            window = window.with_fullscreen(Some(events_loop.get_primary_monitor()));
        } else {
            window = window.with_dimensions(LogicalSize::new(screen_size[0], screen_size[1]));
        }

        let context = ContextBuilder::new().with_vsync(vsync);
        let display = Display::new(window, context, events_loop).unwrap();

        Window {
            display,
            aspect: screen_size[1] as f32 / screen_size[0] as f32,
            open: true,
        }
    }
    pub fn closer(&mut self, event: &Event) {
        match event {
            Event::WindowEvent{ event, .. } => match event {
                WindowEvent::CloseRequested => self.open = false,
                WindowEvent::KeyboardInput{ input, .. } => if let Some(key) = input.virtual_keycode {
                    if key == VirtualKeyCode::Escape && input.state == ElementState::Pressed {
                        self.open = false;
                    }
                }
                _ => (),
            }
            _ => (),
        }
    }
}
