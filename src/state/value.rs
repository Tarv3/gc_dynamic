use imgui::{ImString, Ui};
use util::*;
use state::tool_tips::*;

#[derive(Copy, Clone, Debug)]
pub enum Measurement {
    IsNot,
    Is {
        normalised: [f32; 2],
        init_range: [f32; 2],
        range: [f32; 2],
    },
}

impl Measurement {
    pub fn is_measurement(&self) -> bool {
        match self {
            Measurement::IsNot => false,
            Measurement::Is { .. } => true,
        }
    }
}

#[derive(Debug)]
pub struct Value {
    pub measurement: Measurement,
    pub name: ImString,
    pub tex_indices: Vec<usize>,
    pub selection: f32,
    pub time: bool,
    pub time_updated: bool,
}

impl Value {
    pub fn build_ui_elements(&mut self, ui: &Ui, window_width: f32, hovered: bool) {
        let width = window_width - 110.0;
        if self.tex_indices.len() > 1 {
            ui.with_item_width(width, || {
                self.build_time_ui(ui, width, hovered);
            });
        }
        ui.with_item_width(width, || {
            self.build_measurement_ui(ui, window_width - 50.0, hovered);
        });
    }

    pub fn build_time_ui(&mut self, ui: &Ui, width: f32, hovered: bool) {
        let max = (self.tex_indices.len() as i32) as f32;

        if ui
            .slider_float(im_str!("Time"), &mut self.selection, 0.0, max)
            .build()
        {
            self.time = false;
        }
        time_slider_tt(ui, hovered);

        ui.same_line_spacing(width, 45.0);
        if self.time {
            if ui.button(im_str!("Pause"), (40.0, 20.0)) {
                self.time = false;
            }
            pause_tt(ui, hovered);
        } else {
            if ui.button(im_str!("Play"), (40.0, 20.0)) {
                self.time = true;
            }
            play_tt(ui, hovered);
        }
    }

    pub fn build_measurement_ui(&mut self, ui: &Ui, width: f32, hovered: bool) {
        if let Measurement::Is {
            init_range: [min, max],
            range: ref mut value,
            ..
        } = self.measurement
        {
            let range = max - min;
            let middle = (value[0] + value[1]) * 0.5;
            let min_width = (width * (middle - min) / range).round();
            let min_width = minf32(min_width, 2.0);

            ui.separator();
            ui.text("Range");
            ui.push_item_width(width);
            ui.input_float2(im_str!("##Range floats"), value)
                .decimal_precision(3)
                .build();

            ui.push_item_width(min_width);
            ui.slider_float(im_str!("##Min Range"), &mut value[0], min, middle)
                .build();
            min_range_tt(ui, hovered);

            ui.same_line(min_width + 8.0);
            let max_width = width - min_width;
            let max_width = minf32(max_width, 2.0);
            ui.push_item_width(max_width);
            ui.slider_float(im_str!("##Max Range"), &mut value[1], middle, max)
                .build();
            max_range_tt(ui, hovered);
        }
    }

    pub fn get_textures(&self) -> (usize, usize, f32) {
        let mut ceil = self.selection.ceil() as usize;
        if ceil >= self.tex_indices.len() {
            ceil = 0;
        }
        let mut floor = self.selection.floor();
        if floor >= self.tex_indices.len() as f32 {
            floor = (self.tex_indices.len() - 1) as f32;
        }
        let interpolation = self.selection - floor;

        (
            self.tex_indices[floor as usize],
            self.tex_indices[ceil],
            interpolation,
        )
    }

    pub fn increase_selection(&mut self, dt: f32) {
        if self.tex_indices.len() < 1 {
            return;
        }
        self.selection += dt;
        let max = self.tex_indices.len() as f32;
        if self.selection > max {
            self.selection = self.selection - max;
        }
    }
}
