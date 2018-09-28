use glium::VertexBuffer;
use std::f32::consts::PI;
use util::Vertex;

pub struct Sphere {
    vertical_divs: u32,
    horizontal_divs: u32,
}

impl Sphere {
    pub fn new(v_divs: u32, h_divs: u32) -> Sphere {
        Sphere {
            vertical_divs: v_divs,
            horizontal_divs: h_divs,
        }
    }

    pub fn generate_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        for long in 0..self.vertical_divs {
            let y_tex1 = long as f32 / self.vertical_divs as f32;
            let y_tex2 = (long + 1) as f32 / self.vertical_divs as f32;
            let vert1 = (PI * (y_tex1 + 0.5));
            let vert2 = (PI * (y_tex2 + 0.5));
            let cos_vert1 = vert1.cos();
            let sin_vert1 = vert1.sin();
            let cos_vert2 = vert2.cos();
            let sin_vert2 = vert2.sin();

            for lat in 0..self.horizontal_divs {
                let x_tex1 = lat as f32 / self.horizontal_divs as f32;
                let x_tex2 = (lat + 1) as f32 / self.horizontal_divs as f32;
                let horz1 = (PI * 2.0 * (x_tex1 + 0.5));
                let horz2 = (PI * 2.0 * (x_tex2 + 0.5));
                let sin_horz1 = horz1.sin();
                let cos_horz1 = horz1.cos();
                let sin_horz2 = horz2.sin();
                let cos_horz2 = horz2.cos();

                // Top left
                let x = sin_horz1 * cos_vert1;
                let y = sin_vert1;
                let z = cos_horz1 * cos_vert1;
                let position = [x, y, z];
                let tl = Vertex {
                    position,
                    normal: position,
                    tex_coord: [x_tex1, 1.0 - y_tex1],
                };

                // Bottom left
                let x = sin_horz1 * cos_vert2;
                let y = sin_vert2;
                let z = cos_horz1 * cos_vert2;
                let position = [x, y, z];
                let bl = Vertex {
                    position,
                    normal: position,
                    tex_coord: [x_tex1, 1.0 - y_tex2],
                };

                // Bottom Right
                let x = sin_horz2 * cos_vert2;
                let y = sin_vert2;
                let z = cos_horz2 * cos_vert2;
                let position = [x, y, z];
                let br = Vertex {
                    position,
                    normal: position,
                    tex_coord: [x_tex2, 1.0 - y_tex2],
                };

                // Top Right
                let x = sin_horz2 * cos_vert1;
                let y = sin_vert1;
                let z = cos_horz2 * cos_vert1;
                let position = [x, y, z];
                let tr = Vertex {
                    position,
                    normal: position,
                    tex_coord: [x_tex2, 1.0 - y_tex1],
                };

                vertices.push(tl);
                vertices.push(bl);
                vertices.push(br);

                vertices.push(tl);
                vertices.push(br);
                vertices.push(tr);
            }
        }

        return vertices;
    }
}
