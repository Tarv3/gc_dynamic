mod box_render;
mod tool_tips;
pub mod value;
mod variable;
mod viewports;

use evec::Evec;
use glium;
use glium::index::{NoIndices, PrimitiveType::TrianglesList};
use glium::{
    backend::glutin::Display, draw_parameters::BackfaceCullingMode, texture::Texture2d,
    DrawParameters, Program, Rect, Surface, VertexBuffer,
};
use imgui::*;
use input::MouseState;
use renderer::camera::PCamera;
use sphere::Sphere;
use state::box_render::BoxRenderer;
use state::tool_tips::*;
use state::value::*;
use state::variable::*;
use state::viewports::{DivDirection, Division, VPSettings, ViewPort, ViewRect};
use std::collections::BTreeMap;
use std::error::Error;
use std::mem;
use std::path::Path;
use support::load_image;
use util::*;

pub struct GlobalState {
    pub camera: PCamera,
    zoom: Zoom,
    hsv_program: Program,
    colour_program: Program,

    sphere: VertexBuffer<Vertex>,
    box_renderer: BoxRenderer,

    main_viewport: ViewRect,
    divisions: Evec<Division>,
    viewports: Vec<ViewPort>,
    vp_settings: BTreeMap<usize, VPSettings>,

    height_map: Texture2d,
    overlay: Texture2d,
    textures: Vec<Texture2d>,
    values: Vec<Value>,

    variables: StateVariables,
    menu_width: f32,

    m1_pressed: bool,
    viewport_pressed: Option<usize>,
    mouse_vars: MouseVariables,
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
        box_program: Program,
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

        let sphere = Sphere::new(16, 36);
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
            left: 0.3333,
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
        let mut vp_settings = BTreeMap::new();
        {
            let first_vp = &viewports[0];
            vp_settings.insert(
                first_vp.div_id,
                VPSettings {
                    menu_open: true,
                    show_range: false,
                    cam: None,
                },
            );
        }

        Ok(GlobalState {
            camera,
            zoom,
            hsv_program,
            colour_program,
            sphere: buffer,
            box_renderer: BoxRenderer::new(window, box_program),

            main_viewport,
            divisions,
            viewports,
            vp_settings,

            height_map: load_image(window, height),
            overlay: load_image(window, overlay),
            textures: vec![image],
            values: vec![value],

            variables,
            menu_width: 300.0,
            m1_pressed: false,
            viewport_pressed: None,
            mouse_vars: MouseVariables::new(),
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

    pub fn handle_mouse(&mut self, mouse: &MouseState, dimensions: (u32, u32), hidpi: f32) {
        self.mouse_vars
            .handle_mouse(mouse, self.viewports.as_slice(), dimensions);

        if !mouse.on_ui {
            self.zoom.add_zoom(-mouse.mouse.wheel);
        }
        if let Some(drag) = mouse.get_drag_off_ui() {
            let (x_abs, y_abs) = (drag.x.abs(), drag.y.abs());
            let drag_x = clampf32(self.zoom.get_scale() * drag.x, -x_abs, x_abs);
            let drag_y = clampf32(self.zoom.get_scale() * drag.y, -y_abs, y_abs);
            let drag_speed = self.variables.drag_speed;
            let camera = match self.mouse_vars.viewport_pressed {
                Some(id) => {
                    let div_id = self.viewports[id].div_id;
                    self.get_vp_camera_mut(div_id)
                }
                None => &mut self.camera,
            };
            camera.rotate_around_look_horizontal(-drag_x * drag_speed / hidpi);
            camera.rotate_around_look_vertical(drag_y * drag_speed / hidpi);
        }
    }

    pub fn handle_resize(&mut self, resized: (f64, f64), hidpi: f32) {
        let (x, y) = (resized.0 as f32, resized.1 as f32);
        let new_left = self.menu_width / x / hidpi;
        self.main_viewport.left = new_left;
        self.rebuild_viewports();
        let aspect = x / y;
        self.camera.set_aspect(aspect);
        for settings in self.vp_settings.values_mut() {
            match settings.cam {
                Some(ref mut cam) => cam.set_aspect(aspect),
                _ => (),
            }
        }
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

    pub fn get_vp_camera(&self, div_id: usize) -> &PCamera {
        match self.vp_settings.get(&div_id) {
            Some(ref settings) => match settings.cam {
                Some(ref cam) => &cam,
                None => &self.camera,
            },
            None => &self.camera,
        }
    }

    pub fn get_vp_camera_mut(&mut self, div_id: usize) -> &mut PCamera {
        match self.vp_settings.get_mut(&div_id) {
            Some(settings) => match settings.cam {
                Some(ref mut cam) => cam,
                None => &mut self.camera,
            },
            None => &mut self.camera,
        }
    }

    pub fn vp_has_cam(&self, div_id: usize) -> bool {
        match self.vp_settings.get(&div_id) {
            Some(settings) => settings.cam.is_some(),
            None => false,
        }
    }

    pub fn give_vp_global_cam(&mut self, div_id: usize) {
        match self.vp_settings.get_mut(&div_id) {
            Some(ref mut settings) => {
                settings.cam = Some(self.camera);
            }
            None => panic!("Tried to give camera to a missing viewport"),
        }
    }

    pub fn remove_vp_cam(&mut self, div_id: usize) {
        match self.vp_settings.get_mut(&div_id) {
            Some(ref mut settings) => settings.cam = None,
            None => panic!("Tried to remove camera from a missing viewport"),
        }
    }

    pub fn get_vp_menu_open_mut(&mut self, div_id: usize) -> Option<&mut bool> {
        match self.vp_settings.get_mut(&div_id) {
            Some(settings) => Some(&mut settings.menu_open),
            None => None,
        }
    }

    pub fn get_vp_menu_open(&self, div_id: usize) -> Option<bool> {
        match self.vp_settings.get(&div_id) {
            Some(settings) => Some(settings.menu_open),
            None => None,
        }
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

    pub fn set_values_time(&mut self, val: bool) {
        for value in &mut self.values {
            value.time = val;
        }
    }

    pub fn on_viewport(&self, pos: [f32; 2]) -> Option<usize> {
        for (i, viewport) in self.viewports.iter().enumerate() {
            if viewport.rect.contains(pos) {
                return Some(i);
            }
        }
        None
    }

    pub fn collapse_all(&mut self) {
        if self.viewports.len() <= 1 {
            return;
        }
        let div = self.divisions[0].expect("No first division");

        div.remove_division(&mut self.divisions, 0);
        self.vp_settings.clear();
        self.rebuild_viewports();
    }

    pub fn hide_menus(&mut self) {
        for settings in self.vp_settings.values_mut() {
            settings.menu_open = false;
        }
    }

    pub fn lock_globes(&mut self) {
        for settings in self.vp_settings.values_mut() {
            settings.cam = None;
        }
    }

    pub fn change_projection(&mut self) {}

    pub fn rebuild_viewports(&mut self) {
        let division = self.divisions[0].unwrap();
        let main_viewport = self.main_viewport;
        let mut viewports = vec![];
        mem::swap(&mut viewports, &mut self.viewports);

        viewports.clear();
        division.build_viewports_evec(main_viewport, &self.divisions, &mut viewports);
        let mut settings = BTreeMap::new();

        for viewport in &viewports {
            let id = viewport.div_id;
            let value = self.vp_settings.get(&id);
            match value {
                Some(value) => settings.insert(id, *value),
                None => settings.insert(
                    id,
                    VPSettings {
                        menu_open: true,
                        show_range: false,
                        cam: None,
                    },
                ),
            };
        }
        self.vp_settings = settings;
        mem::swap(&mut viewports, &mut self.viewports);
    }

    pub fn render_viewports<T: Surface + ?Sized>(
        &self,
        target: &mut T,
        model_matrix: [[f32; 4]; 4],
    ) {
        let frame = target.get_dimensions();
        for viewport in &self.viewports {
            if !viewport.can_render() {
                continue;
            }
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

            let id = viewport.div_id;
            let index = self.divisions[id].unwrap().get_selected().unwrap() as usize;
            let settings = self.vp_settings.get(&id).expect("No viewport settings");
            let camera = match settings.cam {
                Some(ref cam) => cam,
                None => &self.camera,
            };

            let view_matrix = viewport.view_matrix(camera, self.zoom.get_scale());
            self.draw_globe(
                index,
                target,
                &draw_parameters,
                *view_matrix.as_ref(),
                model_matrix,
            );

            let draw_parameters = DrawParameters {
                viewport: Some(rect),
                ..Default::default()
            };

            if settings.show_range && self.is_selected_measurement(index).is_measurement() {
                self.box_renderer
                    .render(target, [0.78, 0.0], [0.1, 1.8], &draw_parameters);
            }
        }
    }

    pub fn draw_box<T: Surface + ?Sized>(&self, target: &mut T, rect: Rect) {}

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
                    ).unwrap();
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
                    ).unwrap();
            }
        }
    }

    pub fn viewport_window<F: FnOnce() -> ()>(
        ui: &Ui,
        name: &ImStr,
        pos: (f32, f32),
        rect: glium::Rect,
        width: f32,
        opened: &mut bool,
        func: F,
    ) {
        let x = _maxf32(rect.width as f32, width);
        let y =  _maxf32(rect.height as f32, 400.0);
        ui.window(name)
            .size(
                (x, y),
                ImGuiCond::Always,
            ).position(pos, ImGuiCond::Always)
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .horizontal_scrollbar(true)
            .opened(opened)
            .build(func);
    }

    pub fn closed_window(&mut self, ui: &Ui, pos: (f32, f32), opened: &mut bool) {
        ui.with_color_var(ImGuiCol::WindowBg, [0.0, 0.0, 0.0, 0.0], || {
            ui.window(im_str!(""))
                .position(pos, ImGuiCond::Always)
                .size((50.0, 35.0), ImGuiCond::Always)
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .movable(false)
                .build(|| {
                    if ui.small_button(im_str!("Menu")) {
                        *opened = true;
                    }
                });
        });
    }

    pub fn build_viewport_uis(&mut self, ui: &Ui) {
        let frame_size = ui.frame_size().logical_size;
        let window_width = self.menu_width - 95.0;
        let mut viewports = vec![];
        let mut divisions = Evec::new();
        let mut rebuild = false;

        mem::swap(&mut viewports, &mut self.viewports);
        mem::swap(&mut divisions, &mut self.divisions);

        for (i, viewport) in viewports.iter().enumerate() {
            let rect = viewport.glium_vp_logicalsize(frame_size);
            let x = rect.left as f32;
            let y = frame_size.1 as f32 - (rect.bottom + rect.height) as f32;
            let string = ImString::new(format!("Viewport {}", i));
            let mut opened = self.get_vp_menu_open(viewport.div_id).unwrap_or(false);
            if opened {
                GlobalState::viewport_window(
                    ui, string.as_ref(), (x, y), rect, window_width, &mut opened,
                    || {
                        self.build_viewport_ui(
                            ui,
                            window_width,
                            viewport,
                            &mut divisions,
                            &mut rebuild,
                        );
                    },
                );
            } else {
                self.closed_window(ui, (x, y), &mut opened);
            }
            if let Some(menu) = self.get_vp_menu_open_mut(viewport.div_id) {
                *menu = opened;
            }
        }

        mem::swap(&mut viewports, &mut self.viewports);
        mem::swap(&mut divisions, &mut self.divisions);

        if rebuild {
            self.rebuild_viewports();
        }
    }

    fn build_viewport_ui(
        &mut self,
        ui: &Ui,
        window_width: f32,
        viewport: &ViewPort,
        divisions: &mut Evec<Division>,
        rebuild: &mut bool,
    ) {
        let hovered = self.mouse_vars.hovered;

        ui.text("Split");
        let button_size = (80.0, 40.0);
        let id = viewport.div_id;
        let div = match divisions[id] {
            Some(div) => div,
            None => return,
        };
        if ui.button(im_str!("Vertical"), button_size) {
            div.divide(DivDirection::Verticle(0.5), divisions);
            *rebuild = true;
        }
        vertical_split_tt(ui, hovered);

        ui.same_line_spacing(button_size.0, 12.0);
        if ui.button(im_str!("Horizontal"), button_size) {
            div.divide(DivDirection::Horizontal(0.5), divisions);
            *rebuild = true;
        }
        horizontal_split_tt(ui, hovered);

        if let Some(parent) = div.get_parent() {
            if ui.button(im_str!("Collapse"), button_size) {
                let parent = divisions[parent].unwrap();
                let (a, b) = parent.get_children();
                let pid = parent.get_id();
                parent.remove_division(divisions, div.get_selected().unwrap());

                if let Some(settings) = self.vp_settings.remove(&id) {
                    self.vp_settings.insert(pid, settings);
                }
                match id == a {
                    true => self.vp_settings.remove(&b),
                    false => self.vp_settings.remove(&a),
                };
                *rebuild = true;
            }
            collapse_tt(ui, hovered);
        }

        if !*rebuild {
            if self.vp_has_cam(id) {
                if ui.small_button(im_str!("Lock")) {
                    self.remove_vp_cam(id);
                }
                lock_tt(ui, hovered);
            } else {
                if ui.small_button(im_str!("UnLock")) {
                    self.give_vp_global_cam(id);
                }
                unlock_tt(ui, hovered);

            }

            ui.separator();
            ui.spacing();

            let mut dummy = 0;
            let div = match viewport.get_div_selection_mut(&mut divisions.values) {
                Some(val) => val,
                None => &mut dummy,
            };
            
            self.build_value_selector(ui, window_width, div);
        } else {
            if self.vp_has_cam(id) {
                ui.small_button(im_str!("Lock"));
            } else {
                ui.small_button(im_str!("UnLock"));
            }
            ui.separator();
            ui.spacing();
            let mut dummy = 0;
            let div = match viewport.get_div_selection_mut(&mut divisions.values) {
                Some(val) => val,
                None => &mut dummy,
            };
            self.build_value_selector(ui, window_width, div);
        }
    }

    pub fn build_ui(&mut self, ui: &Ui) {
        let frame_size = ui.frame_size();
        let window_width = self.menu_width;

        ui.window(im_str!("State"))
            .size(
                (window_width, frame_size.logical_size.1 as f32),
                ImGuiCond::Always,
            ).position((0.0, 0.0), ImGuiCond::Always)
            .collapsible(true)
            .movable(false)
            .resizable(false)
            .build(|| {
                let button_size = 100.0;
                let button_y = 30.0;
                self.build_projection_options(ui, (button_size, button_y), 12.0);
                self.variables.build_variable_sliders(ui, button_size * 2.0 + 4.0, self.mouse_vars.hovered);
                self.build_user_buttons(ui, (100.0, 40.0), 12.0);

                ui.text(format!("Fps: {:.2}", ui.framerate()));
            });
        self.build_viewport_uis(ui);
    }

    fn build_user_buttons(&mut self, ui: &Ui, size: (f32, f32), spacing: f32) {
        if ui.button(im_str!("Reset Time"), size) {
            self.reset_time_values();
        }
        ui.same_line_spacing(size.0, spacing);
        if ui.button(im_str!("Lock All"), size) {
            self.lock_globes();
        }
        if ui.button(im_str!("Pause All"), size) {
            self.set_values_time(false);
        }
        ui.same_line_spacing(size.0, spacing);
        if ui.button(im_str!("Play All"), size) {
            self.set_values_time(true);
        }

        if ui.button(im_str!("Collapse All"), size) {
            self.collapse_all();
        }
        ui.same_line_spacing(size.0, spacing);
        if ui.button(im_str!("Close Menus"), size) {
            self.hide_menus();
        }
    }

    fn build_projection_options(&mut self, ui: &Ui, size: (f32, f32), spacing: f32) {
        if ui.button(im_str!("Perspective"), size) {
            self.camera.perspective_projection();
            for settings in self.vp_settings.values_mut() {
                if let Some(ref mut camera) = settings.cam {
                    camera.perspective_projection();
                }
            }
        }
        ui.same_line_spacing(size.0, spacing);
        if ui.button(im_str!("Orthographic"), size) {
            self.camera.orthographic_projection();
            for settings in self.vp_settings.values_mut() {
                if let Some(ref mut camera) = settings.cam {
                    camera.orthographic_projection();
                }
            }
        }
        ui.same_line_spacing(size.0 * 2.0, spacing + 4.0);
        ui.text("Projection");
    }

    fn build_value_selector(&mut self, ui: &Ui, window_width: f32, to_select: &mut i32) {
        ui.text("Select Variable");
        ui.child_frame(im_str!("Select Variable"), (window_width - 30.0, 100.0))
            .movable(false)
            .show_scrollbar_with_mouse(true)
            .show_borders(false)
            .collapsible(true)
            .build(|| {
                for (i, value) in self.values.iter().enumerate() {
                    ui.radio_button(value.name.as_ref(), to_select, i as i32);
                }
            });
        value_selector_tt(ui, self.mouse_vars.hovered);

        let selected = &mut self.values[*to_select as usize];
        ui.separator();
        ui.text(&selected.name);
        selected.build_ui_elements(ui, window_width, self.mouse_vars.hovered);
    }
}
