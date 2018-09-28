use glium::{backend::glutin::Display, texture::Texture2d};
use heat_map;
use std::path::Path;
use support::load_image;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2]
}

implement_vertex!(Vertex, position, normal, tex_coord);

pub fn load_monthly_values(
    display: &Display,
    path: impl AsRef<Path>,
    range: Option<heat_map::math::Range<f32>>,
) -> Vec<Texture2d> {
    let temp_grid: heat_map::grid::Grid<Option<heat_map::data::YearlyData<f32>>> =
        heat_map::grid::Grid::load_from_bin(path).unwrap();
    let mut textures = Vec::with_capacity(12);

    for i in 0..12 {
        let month = temp_grid.into_grid_with(|yearly_temp| match yearly_temp {
            Some(data) => data.get_month_average(i),
            None => None,
        });
        let (texture, _) = month.into_texture(display, range);
        textures.push(texture);
    }
    textures
}

pub fn minf32(value: f32, min: f32) -> f32 {
    if value < min || value.is_nan() || value.is_infinite() {
        min
    } else {
        value
    }
}

pub fn maxf32(value: f32, max: f32) -> f32 {
    if value > max || value.is_nan() || value.is_infinite() {
        max
    } else {
        value
    }
}

pub fn clampf32(value: f32, min: f32, max: f32) -> f32 {
    if value > max || value.is_nan() || value.is_infinite() {
        max
    } else if value < min {
        min
    } else {
        value
    }
}
