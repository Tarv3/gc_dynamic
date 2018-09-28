use glium::index::{NoIndices, PrimitiveType::TrianglesList};
use glium::{
    backend::glutin::Display, texture::Texture2d, DrawParameters, Program, Surface, VertexBuffer,
};
use heat_map;
use imgui::*;
use input::MouseState;
use na;
use renderer::camera::PCamera;
use std::error::Error;
use std::path::Path;
use support::load_image;
use util::*;
use sphere::Sphere;

pub struct Zoom {
    pub rate: f32,
    pub level: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

impl Zoom {
    pub fn add_zoom(&mut self, zoom: f32) {
        self.level += zoom;

        if let Some(max) = self.max {
            if max < self.level {
                self.level = max;
            }
        }
        if let Some(min) = self.min {
            if min > self.level {
                self.level = min;
            }
        }
    }

    pub fn get_scale(&self) -> f32 {
        self.rate.powf(self.level)
    }
}

struct StateVariables {
    pub height: f32,
    pub time_multi: f32,
}

pub struct GlobalState {
    pub camera: PCamera,
    zoom: Zoom,
    hsv_program: Program,
    colour_program: Program,

    sphere: VertexBuffer<Vertex>,

    height_map: Texture2d,
    overlay: Texture2d,
    textures: Vec<Texture2d>,
    values: Vec<Value>,

    selected: i32,
    variables: StateVariables,
}

impl GlobalState {
    pub fn new_default_tex(
        window: &Display,
        camera: PCamera,
        path: impl AsRef<Path>,
        hsv_program: Program,
        colour_program: Program,
    ) -> Result<GlobalState, Box<Error>> {
        let image = load_image(window, path);
        let value = Value {
            measurement: false,
            name: ImString::new("World Map"),
            tex_indices: vec![0],
            selection: 0.0,
            time: false,
        };

        let sphere = Sphere::new(200, 200);
        let verts = sphere.generate_vertices();
        let buffer = VertexBuffer::new(window, &verts)?;
        let zoom = Zoom {
            rate: 1.1,
            level: 0.0,
            max: Some(30.0),
            min: Some(-30.0),
        };

        let variables = StateVariables {
            height: 0.1,
            time_multi: 1.0,
        };

        Ok(GlobalState {
            camera,
            zoom,
            hsv_program,
            colour_program,
            sphere: buffer,
            height_map: load_image(window, "assets/whms.png"),
            overlay: load_image(window, "assets/Pure B and W Map.png"),
            textures: vec![image],
            values: vec![value],
            selected: 0,
            variables,
        })
    }

    pub fn add_new_value(
        &mut self,
        mut new_textures: Vec<Texture2d>,
        name: ImString,
        measurement: bool,
    ) {
        let len = self.textures.len();
        let indices = (len..len + new_textures.len()).collect();
        let value = Value {
            measurement,
            name,
            tex_indices: indices,
            selection: 0.0,
            time: false,
        };

        self.values.push(value);
        self.textures.append(&mut new_textures);
    }

    pub fn handle_mouse(&mut self, mouse: &MouseState) {
        if let Some(drag) = mouse.get_drag_off_ui() {
            let abs_x = drag.x.abs();
            let abs_y = drag.y.abs();
            let drag_x = clampf32(self.zoom.get_scale() * drag.x, -abs_x, abs_x);
            let drag_y = clampf32(self.zoom.get_scale() * drag.y, -abs_y, abs_y);
            self.camera.rotate_around_look_horizontal(-drag_x * 0.01);
            self.camera.rotate_around_look_vertical(drag_y * 0.01);
        }
        if !mouse.on_ui {
            self.zoom.add_zoom(-mouse.mouse.wheel);
        }
    }

    pub fn get_selected(&self) -> Option<&Value> {
        if (self.selected as usize) < self.values.len() {
            return Some(&self.values[self.selected as usize]);
        }
        None
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut Value> {
        if (self.selected as usize) < self.values.len() {
            return Some(&mut self.values[self.selected as usize]);
        }
        None
    }

    pub fn is_selected_measurement(&self) -> bool {
        match self.get_selected() {
            Some(value) => value.measurement,
            None => false,
        }
    }

    pub fn get_selected_textures(&self) -> Option<(&Texture2d, &Texture2d, f32)> {
        let value = match self.get_selected() {
            Some(value) => value,
            None => return None,
        };
        let (i, j, interp) = value.get_selected();
        let len = self.textures.len();
        if i < len && j < len {
            return Some((&self.textures[i], &self.textures[j], interp));
        }
        None
    }

    pub fn update_time(&mut self, dt: f32) {
        let time_multi = self.variables.time_multi;
        let value = match self.get_selected_mut() {
            Some(value) => value,
            None => return,
        };
        if value.time {
            value.increase_selection(dt * time_multi);
        }
    }

    pub fn draw_globe<T: Surface + ?Sized>(
        &self,
        target: &mut T,
        draw_params: &DrawParameters,
        model_matrix: [[f32; 4]; 4],
    ) {
        let (tex1, tex2, interp) = self.get_selected_textures().unwrap();
        let uniforms = uniform! {
            overlay: &self.overlay,
            colour_map1: tex1,
            colour_map2: tex2,
            interpolation: interp,
            view: *self.camera.zoomed_view_matrix(self.zoom.get_scale()).as_ref(),
            eye: *self.camera.position.coords.as_ref(),
            rotation: model_matrix,
            height_map: &self.height_map,
            height_scale: self.variables.height
        };

        if self.is_selected_measurement() {
            target
                .draw(
                    &self.sphere,
                    NoIndices(TrianglesList),
                    &self.hsv_program,
                    &uniforms,
                    draw_params,
                )
                .unwrap();
        } else {
            target
                .draw(
                    &self.sphere,
                    NoIndices(TrianglesList),
                    &self.colour_program,
                    &uniforms,
                    draw_params,
                )
                .unwrap();
        }
    }

    pub fn build_ui(&mut self, ui: &Ui) {
        let frame_size = ui.frame_size();

        ui.window(im_str!("State"))
            .size((300.0, frame_size.logical_size.1 as f32), ImGuiCond::Always)
            .position((0.0, 0.0), ImGuiCond::Always)
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .build(|| {
                let button_size = 100.0;
                let button_y = 30.0;
                self.build_projection_options(ui, (button_size, button_y), 12.0);
                self.build_variable_sliders(ui, button_size * 2.0 + 4.0);
                ui.separator();
                ui.text("Select value to display");
                ui.spacing();
                self.build_value_selector(ui);
            })
    }

    fn build_projection_options(&mut self, ui: &Ui, size: (f32, f32), spacing: f32) {
        if ui.button(im_str!("Perspective"), size) {
            self.camera.perspective_projection();
        }
        ui.same_line_spacing(size.0, spacing);
        if ui.button(im_str!("Orthographic"), size) {
            self.camera.orthographic_projection();
        }
        ui.same_line_spacing(size.0 * 2.0, spacing + 4.0);
        ui.text("Projection");
    }

    fn build_variable_sliders(&mut self, ui: &Ui, width: f32) {
        ui.with_item_width(width, || {
            ui.slider_float(im_str!("Height"), &mut self.variables.height, 0.0, 1.0)
                .build();
            ui.slider_float(
                im_str!("Time Multi"),
                &mut self.variables.time_multi,
                0.0,
                10.0,
            ).build();
        });
    }

    fn build_value_selector(&mut self, ui: &Ui) {
        for (i, value) in self.values.iter().enumerate() {
            ui.radio_button(value.name.as_ref(), &mut self.selected, i as i32);
        }
        let selected = &mut self.values[self.selected as usize];
        ui.spacing();
        ui.separator();
        ui.text(&selected.name);
        selected.build_ui_elements(ui);
    }
}

struct Value {
    measurement: bool,
    name: ImString,
    tex_indices: Vec<usize>,
    selection: f32,
    time: bool,
}

impl Value {
    pub fn build_ui_elements(&mut self, ui: &Ui) {
        if self.tex_indices.len() > 1 {
            let width = 200.0;
            let max = (self.tex_indices.len() as i32 - 1) as f32;
            ui.with_item_width(width, || {
                if ui.slider_float(im_str!("Time"), &mut self.selection, 0.0, max)
                    .build()
                {
                    self.time = false;
                }
            });
            ui.same_line_spacing(width, 45.0);
            if self.time {
                if ui.button(im_str!("Pause"), (40.0, 20.0)) {
                    self.time = false;
                }
            } else {
                if ui.button(im_str!("Play"), (40.0, 20.0)) {
                    self.time = true;
                }
            }
        }
    }

    pub fn get_selected(&self) -> (usize, usize, f32) {
        let mut ceil = self.selection.ceil() as usize;
        if ceil as usize >= self.tex_indices.len() {
            ceil -= 1;
        }
        let floor = self.selection.floor();
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
        let max = self.tex_indices.len() as f32 - 1.0;
        if self.selection > max {
            self.selection = self.selection - max;
        }
    }
}
