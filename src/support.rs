use glium::backend::glutin::Display;
use glium::texture::{texture2d::Texture2d, RawImage2d};
use glium::{
    glutin::{Event, EventsLoop}, Frame,
};
use image;
use imgui::*;
use imgui::{FrameSize, ImGui, Ui, ImVec4};
use imrender::Renderer;
use input::MouseState;
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

    (imgui, imrender)
}

fn get_time_and_reset(instant: &mut Instant) -> f32 {
    let now = Instant::now();
    let delta = now - *instant;
    *instant = now;

    delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0
}

pub fn run<F>(window: &mut Window, events_loop: &mut EventsLoop, mut func: F)
where
    F: FnMut(&mut Frame, &Ui, &MouseState, &Vec<Event>, f32) -> bool,
{
    let hidpi_factor = window.display.gl_window().get_hidpi_factor().round();
    let (mut imgui, mut renderer) = build_imgui(window, hidpi_factor as f32);
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

        let frame_size = FrameSize {
            logical_size: physical_size.to_logical(hidpi_factor).into(),
            hidpi_factor,
        };

        mouse.update_mouse(&mut imgui);
        mouse.reset();
        let ui = imgui.frame(frame_size, delta_s);

        events.clear();
        events_loop.poll_events(|event| {
            window.closer(&event);
            mouse.handle_window_event(&event, &ui, &window.display.gl_window(), hidpi_factor);
            events.push(event);
        });
        mouse.update_on_ui(&ui);

        let mut target = window.display.draw();
        if !func(&mut target, &ui, &mouse, &events, delta_s) {
            break;
        }

        renderer.render(&mut target, ui).expect("Failed to render");
        target.finish().unwrap();
    }
}

