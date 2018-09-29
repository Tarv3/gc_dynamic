use glium::{backend::glutin::Display, texture::Texture2d};
use heat_map;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coord);

pub fn load_monthly_values(
    display: &Display,
    path: impl AsRef<Path>,
    range: heat_map::math::Range<f32>,
) -> (Vec<Texture2d>, Vec<Texture2d>) {
    let temp_grid: heat_map::grid::Grid<Option<heat_map::data::YearlyData<f32>>> =
        heat_map::grid::Grid::load_from_bin(path).unwrap();
    let mut textures = Vec::with_capacity(12);

    for i in 0..12 {
        let month = temp_grid.into_grid_with(|yearly_temp| match yearly_temp {
            Some(data) => data.get_month_average(i),
            None => None,
        });
        let (texture, _) = month.into_texture(display, Some(range));
        textures.push(texture);
    }
    let avg_temp = vec![
        temp_grid
            .into_grid_with(|temp| match temp {
                Some(data) => data.yearly_average(),
                None => None,
            })
            .into_texture(display, Some(range)).0,
    ];
    (avg_temp, textures)
}

pub fn minf32(value: f32, min: f32) -> f32 {
    if value < min || value.is_nan() || value.is_infinite() {
        min
    } else {
        value
    }
}

pub fn _maxf32(value: f32, max: f32) -> f32 {
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
