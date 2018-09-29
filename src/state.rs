use evec::Evec;
use glium;
use glium::index::{NoIndices, PrimitiveType::TrianglesList};
use glium::{
    backend::glutin::Display, draw_parameters::BackfaceCullingMode, texture::Texture2d,
    DrawParameters, Program, Surface, VertexBuffer,
};
use imgui::*;
use input::MouseState;
use renderer::camera::PCamera;
use sphere::Sphere;
use std::error::Error;
use std::mem;
use std::path::Path;
use support::load_image;
use util::*;
use viewports::{DivDirection, Division, ViewPort, ViewRect};

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
    pub drag_speed: f32,
}

pub struct GlobalState {
    pub camera: PCamera,
    zoom: Zoom,
    hsv_program: Program,
    colour_program: Program,

    sphere: VertexBuffer<Vertex>,

    main_viewport: ViewRect,
    divisions: Evec<Division>,
    viewports: Vec<ViewPort>,
    division_order: Vec<usize>,

    height_map: Texture2d,
    overlay: Texture2d,
    textures: Vec<Texture2d>,
    values: Vec<Value>,

    selected: i32,
    variables: StateVariables,
    menu_width: f32,
}

impl GlobalState {
    pub fn new_default_tex(
        window: &Display,
        camera: PCamera,
        height: impl AsRef<Path>,
        overlay: impl AsRef<Path>,
        tex_name: ImString,
        path: impl AsRef<Path>,
        hsv_program: Program,
        colour_program: Program,
    ) -> Result<GlobalState, Box<Error>> {
        let image = load_image(window, path);
        let value = Value {
            measurement: Measurement::IsNot,
            name: tex_name,
            tex_indices: vec![0],
            selection: 0.0,
            time: false,
            time_updated: false,
        };

        let sphere = Sphere::new(250, 250);
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
            drag_speed: 0.02,
        };
        let main_viewport = ViewRect {
            left: 0.33,
            right: 1.0,
            bottom: 0.0,
            top: 1.0,
        };
        let division = Division::None {
            selected: 0,
            id: 0,
            parent: None,
        };
        let divisions = Evec::from(vec![division]);

        let mut viewports = vec![];
        division.build_viewports_evec(main_viewport, &divisions, &mut viewports);

        Ok(GlobalState {
            camera,
            zoom,
            hsv_program,
            colour_program,
            sphere: buffer,

            main_viewport,
            divisions,
            viewports,
            division_order: vec![],

            height_map: load_image(window, height),
            overlay: load_image(window, overlay),
            textures: vec![image],
            values: vec![value],

            selected: 0,
            variables,
            menu_width: 300.0,
        })
    }

    pub fn add_new_value(
        &mut self,
        mut new_textures: Vec<Texture2d>,
        name: ImString,
        measurement: Measurement,
    ) {
        let len = self.textures.len();
        let indices = (len..len + new_textures.len()).collect();
        let value = Value {
            measurement,
            name,
            tex_indices: indices,
            selection: 0.0,
            time: false,
            time_updated: false,
        };

        self.values.push(value);
        self.textures.append(&mut new_textures);
    }

    pub fn handle_mouse(&mut self, mouse: &MouseState, hidpi: f32) {
        if let Some(drag) = mouse.get_drag_off_ui() {
            let abs_x = drag.x.abs();
            let abs_y = drag.y.abs();
            let drag_x = clampf32(self.zoom.get_scale() * drag.x, -abs_x, abs_x);
            let drag_y = clampf32(self.zoom.get_scale() * drag.y, -abs_y, abs_y);
            self.camera
                .rotate_around_look_horizontal(-drag_x * self.variables.drag_speed * hidpi);
            self.camera
                .rotate_around_look_vertical(drag_y * self.variables.drag_speed * hidpi);
        }
        if !mouse.on_ui {
            self.zoom.add_zoom(-mouse.mouse.wheel);
        }
    }

    pub fn handle_resize(&mut self, resized: (f64, f64)) {
        let (x, y) = (resized.0 as f32, resized.1 as f32);
        let new_left = self.menu_width / x;
        self.main_viewport.left = new_left;
        self.rebuild_viewports();
    }

    fn get_selected(&self, index: usize) -> Option<&Value> {
        if index < self.values.len() {
            return Some(&self.values[index]);
        }
        None
    }

    fn get_selected_mut(&mut self, index: usize) -> Option<&mut Value> {
        if index < self.values.len() {
            return Some(&mut self.values[index]);
        }
        None
    }

    pub fn is_selected_measurement(&self, index: usize) -> Measurement {
        match self.get_selected(index) {
            Some(value) => value.measurement,
            None => Measurement::IsNot,
        }
    }

    pub fn get_selected_textures(&self, index: usize) -> Option<(&Texture2d, &Texture2d, f32)> {
        let value = match self.get_selected(index) {
            Some(value) => value,
            None => return None,
        };
        let (i, j, interp) = value.get_textures();
        let len = self.textures.len();
        if i < len && j < len {
            return Some((&self.textures[i], &self.textures[j], interp));
        }
        None
    }

    pub fn update_time(&mut self, dt: f32) {
        self.reset_time_updates();
        let time_multi = self.variables.time_multi;
        let mut viewports = vec![];
        mem::swap(&mut viewports, &mut self.viewports);
        for viewport in &viewports {
            let index = match viewport.get_div_selection(&self.divisions.values) {
                Some(index) => index as usize,
                None => continue,
            };
            let value = match self.get_selected_mut(index) {
                Some(value) => value,
                None => return,
            };
            if value.time && !value.time_updated {
                value.increase_selection(dt * time_multi);
                value.time_updated = true;
            }
        }
        mem::swap(&mut viewports, &mut self.viewports);
    }

    pub fn reset_time_updates(&mut self) {
        for value in &mut self.values {
            value.time_updated = false;
        }
    }

    pub fn reset_time_values(&mut self) {
        for value in &mut self.values {
            value.selection = 0.0;
        }
    }

    pub fn rebuild_viewports(&mut self) {
        let division = self.divisions[0].unwrap();
        let main_viewport = self.main_viewport;
        let mut viewports = vec![];
        mem::swap(&mut viewports, &mut self.viewports);
        viewports.clear();
        division.build_viewports_evec(main_viewport, &self.divisions, &mut viewports);
        mem::swap(&mut viewports, &mut self.viewports);
    }

    pub fn render_viewports<T: Surface + ?Sized>(
        &self,
        target: &mut T,
        model_matrix: [[f32; 4]; 4],
    ) {
        let frame = target.get_dimensions();
        for ref viewport in self.viewports.iter() {
            let rect = viewport.glium_viewport(frame);
            let draw_parameters = DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                backface_culling: BackfaceCullingMode::CullClockwise,
                viewport: Some(rect),
                ..Default::default()
            };

            let index = viewport.div_id;
            let index = self.divisions[index].unwrap().get_selected() as usize;
            let view_matrix = viewport.view_matrix(&self.camera, self.zoom.get_scale());
            self.draw_globe(
                index,
                target,
                &draw_parameters,
                *view_matrix.as_ref(),
                model_matrix,
            );
        }
    }

    pub fn draw_globe<T: Surface + ?Sized>(
        &self,
        index: usize,
        target: &mut T,
        draw_params: &DrawParameters,
        view_matrix: [[f32; 4]; 4],
        model_matrix: [[f32; 4]; 4],
    ) {
        let (tex1, tex2, interp) = self.get_selected_textures(index).unwrap();
        match self.is_selected_measurement(index) {
            Measurement::Is {
                normalised,
                init_range,
                range,
            } => {
                let uniforms = uniform! {
                    overlay: &self.overlay,
                    colour_map1: tex1,
                    colour_map2: tex2,
                    interpolation: interp,
                    init_range: init_range,
                    range: range,
                    normalised: normalised,
                    view: view_matrix,
                    eye: *self.camera.position.coords.as_ref(),
                    rotation: model_matrix,
                    height_map: &self.height_map,
                    height_scale: self.variables.height
                };
                target
                    .draw(
                        &self.sphere,
                        NoIndices(TrianglesList),
                        &self.hsv_program,
                        &uniforms,
                        draw_params,
                    )
                    .unwrap();
            }
            Measurement::IsNot => {
                let uniforms = uniform! {
                    overlay: &self.overlay,
                    colour_map1: tex1,
                    view: view_matrix,
                    eye: *self.camera.position.coords.as_ref(),
                    rotation: model_matrix,
                    height_map: &self.height_map,
                    height_scale: self.variables.height
                };
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
    }

    pub fn build_viewport_uis(&mut self, ui: &Ui) {
        let frame_size = ui.frame_size().logical_size;
        let window_width = self.menu_width - 100.0;
        let mut viewports = vec![];
        let mut divisions = Evec::new();
        let mut rebuild = false;
        mem::swap(&mut viewports, &mut self.viewports);
        mem::swap(&mut divisions, &mut self.divisions);
        for (i, viewport) in viewports.iter().enumerate() {
            let rect = viewport.glium_vp_logicalsize(frame_size);
            let string = ImString::new(format!("Viewport {}", i));
            ui.window(string.as_ref())
                .size(
                    (window_width, _maxf32(rect.height as f32, 400.0)),
                    ImGuiCond::Always,
                )
                .position(
                    (
                        rect.left as f32,
                        frame_size.1 as f32 - (rect.bottom + rect.height) as f32,
                    ),
                    ImGuiCond::Always,
                )
                .collapsible(true)
                .movable(false)
                .resizable(false)
                .build(|| {
                    ui.text("Split");
                    let button_size = (80.0, 40.0);
                    let vert = ImString::new(format!("Vertical ##{}", i));
                    let horizontal = ImString::new(format!("Horizontal ##{}", i));
                    let div = match divisions[viewport.div_id] {
                        Some(div) => div,
                        None => return,
                    };
                    if ui.button(vert.as_ref(), button_size) {
                        div.divide(DivDirection::Verticle(0.5), &mut divisions);
                        self.division_order.push(div.get_id());
                        rebuild = true;
                    }
                    ui.same_line_spacing(button_size.0, 12.0);
                    if ui.button(horizontal.as_ref(), button_size) {
                        div.divide(DivDirection::Horizontal(0.5), &mut divisions);
                        self.division_order.push(div.get_id());
                        rebuild = true;
                    }
                    if let Some(parent) = div.get_parent() {
                        let collapse = ImString::new(format!("Collapse ##{}", i));
                        if ui.button(collapse.as_ref(), button_size) {
                            let parent = divisions[parent].unwrap();
                            parent.remove_division(&mut divisions, div.get_selected());
                            rebuild = true;
                        }
                    }

                    if !rebuild {
                        ui.separator();
                        ui.text("Select value to display");
                        ui.spacing();
                        let div = match viewport.get_div_selection_mut(&mut divisions.values) {
                            Some(val) => val,
                            None => return,
                        };
                        self.build_value_selector(ui, window_width, div);
                    }
                });
        }
        mem::swap(&mut viewports, &mut self.viewports);
        mem::swap(&mut divisions, &mut self.divisions);

        if rebuild {
            self.rebuild_viewports();
        }
    }

    pub fn build_ui(&mut self, ui: &Ui) {
        let frame_size = ui.frame_size();
        let window_width = self.menu_width;
        let mut hide = false;
        ui.window(im_str!("State"))
            .size(
                (window_width, frame_size.logical_size.1 as f32),
                ImGuiCond::Always,
            )
            .position((0.0, 0.0), ImGuiCond::Always)
            .collapsible(true)
            .movable(false)
            .resizable(false)
            .build(|| {
                let button_size = 100.0;
                let button_y = 30.0;
                self.build_projection_options(ui, (button_size, button_y), 12.0);
                self.build_variable_sliders(ui, button_size * 2.0 + 4.0);
                if ui.button(im_str!("Reset Time"), (100.0, 40.0)) {
                    self.reset_time_values();
                }
                ui.same_line_spacing(100.0, 12.0);
            });
        self.build_viewport_uis(ui);
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
            ui.slider_float(
                im_str!("Drag Speed"),
                &mut self.variables.drag_speed,
                0.005,
                0.08,
            ).build();
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

    fn build_value_selector(&mut self, ui: &Ui, window_width: f32, to_select: &mut i32) {
        for (i, value) in self.values.iter().enumerate() {
            ui.radio_button(value.name.as_ref(), to_select, i as i32);
        }
        let selected = &mut self.values[*to_select as usize];
        ui.spacing();
        ui.separator();
        ui.text(&selected.name);
        selected.build_ui_elements(ui, window_width);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Measurement {
    IsNot,
    Is {
        normalised: [f32; 2],
        init_range: [f32; 2],
        range: [f32; 2],
    },
}

struct Value {
    measurement: Measurement,
    name: ImString,
    tex_indices: Vec<usize>,
    selection: f32,
    time: bool,
    time_updated: bool,
}

impl Value {
    pub fn build_ui_elements(&mut self, ui: &Ui, window_width: f32) {
        let width = window_width - 100.0;
        if self.tex_indices.len() > 1 {
            ui.with_item_width(width, || {
                self.build_time_ui(ui, width);
            });
        }
        ui.with_item_width(width, || {
            self.build_measurement_ui(ui, window_width - 50.0);
        });
    }

    pub fn build_time_ui(&mut self, ui: &Ui, width: f32) {
        let max = (self.tex_indices.len() as i32) as f32;

        if ui.slider_float(im_str!("Time"), &mut self.selection, 0.0, max)
            .build()
        {
            self.time = false;
        }
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

    pub fn build_measurement_ui(&mut self, ui: &Ui, width: f32) {
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
            ui.text("Temperature Range");
            ui.push_item_width(width);
            ui.input_float2(im_str!("##Range floats"), value)
                .decimal_precision(3)
                .build();
            ui.push_item_width(min_width);
            ui.slider_float(im_str!("##Min Range"), &mut value[0], min, middle)
                .build();

            ui.same_line(min_width + 8.0);
            let max_width = width - min_width;
            let max_width = minf32(max_width, 2.0);
            ui.push_item_width(max_width);
            ui.slider_float(im_str!("##Max Range"), &mut value[1], middle, max)
                .build();
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
