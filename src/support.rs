use glium::backend::glutin::Display;
use glium::texture::{texture2d::Texture2d, RawImage2d};
use glium::{
    glutin::{ElementState, Event, EventsLoop, VirtualKeyCode, WindowEvent},
    Frame,
};
use image;
use imgui::{FrameSize, ImGui, ImGuiKey, Ui};
use imrender::Renderer;
use input::MouseState;
use std::error::Error;
use std::path::Path;
use std::time::Instant;
use window::Window;

pub fn load_image(display: &Display, path: impl AsRef<Path>) -> Texture2d {
    let image = image::open(path).expect("Cannot open image").to_rgba();
    let dims = image.dimensions();
    let raw = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dims);
    Texture2d::new(display, raw).unwrap()
}

fn build_imgui(window: &Window, hidpi_factor: f32) -> (ImGui, Renderer) {
    let mut imgui = ImGui::init();
    let imrender =
        Renderer::init(&mut imgui, &window.display).expect("Failed to create imrenderer");
    imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);
    imgui.set_ini_filename(None);

    (imgui, imrender)
}

fn get_time_and_reset(instant: &mut Instant) -> f32 {
    let now = Instant::now();
    let delta = now - *instant;
    *instant = now;

    delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0
}

fn handle_received_char(imgui: &mut ImGui, event: Event) -> Option<Event> {
    let used = match &event {
        Event::WindowEvent { ref event, .. } => match event {
            WindowEvent::ReceivedCharacter(val) => {
                imgui.add_input_character(*val);
                true
            }
            _ => false,
        },
        _ => false,
    };

    if !used {
        Some(event)
    } else {
        None
    }
}

fn handle_special_keys(imgui: &mut ImGui, event: &Event) {
    match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput { input, .. } => {
                match (input.virtual_keycode, input.state == ElementState::Pressed) {
                    (Some(VirtualKeyCode::Back), pressed) => imgui.set_key(0, pressed),
                    (Some(VirtualKeyCode::Delete), pressed) => imgui.set_key(1, pressed),
                    (Some(VirtualKeyCode::Return), pressed) => imgui.set_key(2, pressed),
                    (Some(VirtualKeyCode::Tab), pressed) => imgui.set_key(3, pressed),
                    (Some(VirtualKeyCode::LShift), pressed) => imgui.set_key_shift(pressed),
                    (Some(VirtualKeyCode::LControl), pressed) => imgui.set_key_ctrl(pressed),
                    _ => (),
                }
            }
            _ => (),
        },
        _ => (),
    }
}

fn set_special_keys(imgui: &mut ImGui) {
    imgui.set_imgui_key(ImGuiKey::Backspace, 0);
    imgui.set_imgui_key(ImGuiKey::Delete, 1);
    imgui.set_imgui_key(ImGuiKey::Enter, 2);
    imgui.set_imgui_key(ImGuiKey::Tab, 3);
}

pub fn run<F>(window: &mut Window, events_loop: &mut EventsLoop, mut func: F)
where
    F: FnMut(&mut Frame, &Ui, &MouseState, &Vec<Event>, f32, bool) -> bool,
{
    let hdp = window.display.gl_window().get_hidpi_factor();
    let hidpi_factor = hdp.round();
    let (mut imgui, mut renderer) = build_imgui(window, hdp as f32);
    set_special_keys(&mut imgui);
    let mut mouse = MouseState::new();
    let mut last_frame = Instant::now();
    let mut events = vec![];

    while window.open {
        let delta_s = get_time_and_reset(&mut last_frame);

        let physical_size = window
            .display
            .gl_window()
            .get_inner_size()
            .expect("Failed to get inner size")
            .to_physical(window.display.gl_window().get_hidpi_factor());
        let no_render = physical_size.width == 0.0 || physical_size.height == 0.0;
        let frame_size = FrameSize {
            logical_size: physical_size.to_logical(hidpi_factor).into(),
            hidpi_factor,
        };

        mouse.update_mouse(&mut imgui, delta_s);
        mouse.reset();

        events.clear();
        events_loop.poll_events(|event| {
            window.closer(&event);
            handle_special_keys(&mut imgui, &event);
            if let Some(event) = handle_received_char(&mut imgui, event) {
                events.push(event);
            }
        });

        let ui = imgui.frame(frame_size, delta_s);
        for event in &events {
            mouse.handle_window_event(&event, &ui, &window.display.gl_window(), hidpi_factor);
        }
        mouse.update_on_ui(&ui);

        let mut target = window.display.draw();
        if !func(&mut target, &ui, &mouse, &events, delta_s, no_render) {
            break;
        }
        if no_render {
            ui.render(|_, _| -> Result<(), Box<Error>> { Ok(()) })
                .expect("Failed to render");
        } else {
            renderer.render(&mut target, ui).expect("Failed to render");
        }
        target.finish().unwrap();
    }
}
