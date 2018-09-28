#[macro_use]
extern crate glium;
extern crate image;
#[macro_use]
extern crate imgui;
extern crate heat_map;
extern crate imgui_glium_renderer as imrender;
extern crate nalgebra as na;
extern crate renderer;

mod input;
mod state;
mod support;
mod window;
mod util;
mod sphere;

use glium::backend::glutin::Display;
use glium::index::{NoIndices, PrimitiveType::TrianglesList};
use glium::texture::{texture2d::Texture2d, RawImage2d};
use glium::uniforms::EmptyUniforms;
use glium::{
    draw_parameters::BackfaceCullingMode, glutin::EventsLoop, DrawParameters, Program, Surface,
    VertexBuffer,
};
use heat_map::math::Range;
use imgui::ImString;
use renderer::{
    camera::{PCamera, Projection}, test, Vec3, PV,
};
use state::GlobalState;
use std::f32::consts::PI;
use std::path::Path;
use support::load_image;
use window::Window;
use util::*;
use sphere::Sphere;

fn main() {
    let mut events_loop = EventsLoop::new();
    let mut window = Window::new(false, true, true, [900.0, 900.0], &events_loop);
    let mut camera = PCamera::new(
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Projection::Perspective(PV::new(1.0, PI * 0.25, 0.1, 10.0)),
    );
    let sphere = Sphere::new(200, 200);
    let verts = sphere.generate_vertices();

    let buffer = VertexBuffer::new(&window.display, &verts).unwrap();

    let hsv_program = Program::from_source(
        &window.display,
        include_str!("shaders/vert.glsl"),
        include_str!("shaders/frag_hsv.glsl"),
        None,
    ).unwrap();

    let colour_program = Program::from_source(
        &window.display,
        include_str!("shaders/vert.glsl"),
        include_str!("shaders/frag.glsl"),
        None,
    ).unwrap();

    let draw_parameters = DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        backface_culling: BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };
    let map = load_image(&window.display, "assets/AvgTemp -20_30.png");
    let monthly_values = load_monthly_values(
        &window.display,
        "assets/tempgrid.bin",
        Some(Range::new(-20.0, 30.0)),
    );

    let mut glstate = GlobalState::new_default_tex(
        &window.display,
        camera,
        "assets/map_pic.jpg",
        hsv_program,
        colour_program,
    ).unwrap();

    glstate.add_new_value(vec![map], ImString::new("Average Temperature"), false);
    glstate.add_new_value(monthly_values, ImString::new("Monthly Temperature"), true);

    let identity: na::Matrix4<f32> = na::Matrix4::identity();

    support::run(
        &mut window,
        &mut events_loop,
        |target, ui, mouse, events, dt| {
            glstate.update_time(dt);
            glstate.build_ui(ui);
            glstate.handle_mouse(mouse);
            for event in events {
                input::handle_input(event, &mut glstate.camera);
            }
            
            target.clear_color_and_depth((1.0, 1.0, 1.0, 0.0), 1.0);
            glstate.draw_globe(target, &draw_parameters, *identity.as_ref());
            true
        },
    );
}
