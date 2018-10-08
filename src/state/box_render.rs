use glium::index::{NoIndices, PrimitiveType::TriangleStrip};
use glium::{backend::glutin::Display, DrawParameters, Program, Surface, VertexBuffer};
use util::{build_box, BoxVertex};

pub struct BoxRenderer {
    pub program: Program,
    pub buffer: VertexBuffer<BoxVertex>,
}

impl BoxRenderer {
    pub fn new(display: &Display, program: Program) -> BoxRenderer {
        let buffer = build_box(display, 1.0, 1.0);
        BoxRenderer { program, buffer }
    }

    pub fn render<T: Surface + ?Sized>(&self, target: &mut T, translation: impl Into<[f32; 2]>, scale: impl Into<[f32;2]>, draw_params: &DrawParameters) {
        let uniforms = uniform!{
            translation: translation.into(),
            scale: scale.into(),
        };

        target.draw(
            &self.buffer,
            NoIndices(TriangleStrip),
            &self.program,
            &uniforms,
            draw_params,
        ).unwrap();
    }
}
