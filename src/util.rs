use glium::{backend::glutin::Display, texture::Texture2d, VertexBuffer};
use heat_map;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coord);

#[derive(Copy, Clone, Debug)]
pub struct BoxVertex {
    pub position: [f32; 2],
    pub hue: f32,
}

implement_vertex!(BoxVertex, position, hue);

pub fn build_box(display: &Display, width: f32, height: f32) -> VertexBuffer<BoxVertex> {
    let right = width * 0.5;
    let left = -right;
    let top = height * 0.5;
    let bottom = -top;

    let vals = [
        BoxVertex {
            position: [left, bottom],
            hue: 0.0,
        },
        BoxVertex {
            position: [right, bottom],
            hue: 0.0,
        },
        BoxVertex {
            position: [left, top],
            hue: 1.0,
        },
        BoxVertex {
            position: [right, top],
            hue: 1.0,
        },
    ];

    VertexBuffer::new(display, &vals).unwrap()
}

pub fn load_temp_grid(
    path: impl AsRef<Path>,
) -> heat_map::grid::Grid<Option<heat_map::data::YearlyData<f32>>> {
    heat_map::grid::Grid::load_from_bin(path).unwrap()
}

pub fn load_temp_values(
    display: &Display,
    path: impl AsRef<Path>,
    range: heat_map::math::Range<f32>,
    std_range: heat_map::math::Range<f32>,
) -> (Vec<Texture2d>, Vec<Texture2d>, Vec<Texture2d>) {
    let temp_grid: heat_map::grid::Grid<Option<heat_map::data::YearlyData<f32>>> =
        heat_map::grid::Grid::load_from_bin(path).unwrap();

    let monthly_temps = load_monthly_values(display, &temp_grid, range);
    let avg_temp = vec![load_yearly_average(display, &temp_grid, range)];
    let stddev = vec![load_yearly_stddev(display, &temp_grid, std_range)];
    (avg_temp, monthly_temps, stddev)
}

pub fn load_monthly_values(
    display: &Display,
    temp_grid: &heat_map::grid::Grid<Option<heat_map::data::YearlyData<f32>>>,
    range: heat_map::math::Range<f32>,
) -> Vec<Texture2d> {
    let mut textures = Vec::with_capacity(12);

    for i in 0..12 {
        let month = temp_grid.into_grid_with(|yearly_temp| match yearly_temp {
            Some(data) => data.get_month_average(i),
            None => None,
        });
        let (texture, _) = month.into_texture(display, Some(range));
        textures.push(texture);
    }
    textures
}

pub fn load_yearly_average(
    display: &Display,
    temp_grid: &heat_map::grid::Grid<Option<heat_map::data::YearlyData<f32>>>,
    range: heat_map::math::Range<f32>,
) -> Texture2d {
    temp_grid
        .into_grid_with(|temp| match temp {
            Some(data) => data.yearly_average(),
            None => None,
        }).into_texture(display, Some(range))
        .0
}

pub fn load_yearly_stddev(
    display: &Display,
    temp_grid: &heat_map::grid::Grid<Option<heat_map::data::YearlyData<f32>>>,
    range: heat_map::math::Range<f32>,
) -> Texture2d {
    temp_grid
        .into_grid_with(|temp| match temp {
            Some(data) => data.standard_dev(),
            None => None,
        }).into_texture(display, Some(range))
        .0
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
