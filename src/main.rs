// #![windows_subsystem = "windows"]
#![allow(dead_code)]

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
mod sphere;
mod state;
mod support;
mod util;
mod window;
mod evec;

use glium::backend::glutin::Display;
use glium::{
    draw_parameters::BackfaceCullingMode, glutin::EventsLoop, DrawParameters, Program, Surface,
};
use heat_map::math::Range;
use imgui::ImString;
use renderer::{
    camera::{PCamera, Projection}, Vec3, PV,
};
use state::{GlobalState, Measurement};
use std::f32::consts::PI;
use util::*;
use window::Window;
use support::load_image;

fn build_state(display: &Display) -> GlobalState {
    let camera = PCamera::new(
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Projection::Perspective(PV::new(1.0, PI * 0.25, 0.1, 10.0)),
    );

    let hsv_program = Program::from_source(
        display,
        include_str!("shaders/vert.glsl"),
        include_str!("shaders/frag_hsv.glsl"),
        None,
    ).unwrap();

    let colour_program = Program::from_source(
        display,
        include_str!("shaders/vert.glsl"),
        include_str!("shaders/frag.glsl"),
        None,
    ).unwrap();

    let monthly_range = [-40.0, 50.0];
    let stdrange = [0.0, 40.0];
    let (avg, monthly_values, stddev) = load_temp_values(
        display,
        "assets/tempgrid.bin",
        Range::new(monthly_range[0], monthly_range[1]),
        Range::new(stdrange[0], stdrange[1]),
    );

    let mut glstate = GlobalState::new_default_tex(
        display,
        camera,
        "assets/whms.png",
        "assets/Pure B and W Map.png",
        ImString::new("World Map"),
        "assets/map_pic.jpg",
        hsv_program,
        colour_program,
    ).unwrap();
    let height = vec![load_image(display, "assets/whms.png")];

    glstate.add_new_value(
        height,
        ImString::new("Height"),
        Measurement::Is {
            normalised: [0.0, 1.0],
            init_range: [0.0, 1.0],
            range: [0.0, 1.0]
        },
    );
    
    glstate.add_new_value(
        avg,
        ImString::new("Average Temperature"),
        Measurement::Is {
            normalised: [0.5, 1.0],
            init_range: monthly_range,
            range: monthly_range,
        },
    );

    glstate.add_new_value(
        monthly_values,
        ImString::new("Monthly Temperature"),
        Measurement::Is {
            normalised: [0.5, 1.0],
            init_range: monthly_range,
            range: monthly_range,
        },
    );

    glstate.add_new_value(
        stddev,
        ImString::new("Standard Deviation"),
        Measurement::Is {
            normalised: [0.5, 1.0],
            init_range: stdrange,
            range: stdrange,
        },
    );

    glstate
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let mut window = Window::new(false, true, true, [900.0, 900.0], &events_loop);
    let hidpi = window.display.gl_window().get_hidpi_factor() as f32;
    
    let mut glstate = build_state(&window.display);
    let identity: na::Matrix4<f32> = na::Matrix4::identity();

    let draw_parameters = DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        backface_culling: BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };

    support::run(
        &mut window,
        &mut events_loop,
        |target, ui, mouse, events, dt| {
            let dims = target.get_dimensions();
            glstate.update_time(dt);
            glstate.build_ui(ui);
            glstate.handle_mouse(mouse, dims, hidpi);
            for event in events {
                input::handle_input(event, &mut glstate.camera);
                if let Some(resized) = input::get_resized(event) {
                    glstate.handle_resize(resized, hidpi);
                }
            }

            target.clear_color_and_depth((1.0, 1.0, 1.0, 0.0), 1.0);
            glstate.render_viewports(target, *identity.as_ref());
            true
        },
    );
}
