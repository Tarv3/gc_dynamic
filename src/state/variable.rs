use state::viewports::ViewPort;
use input::MouseState;
use imgui::Ui;
use state::tool_tips::*;

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

pub struct StateVariables {
    pub height: f32,
    pub time_multi: f32,
    pub drag_speed: f32,
}

impl StateVariables {
    pub fn build_variable_sliders(&mut self, ui: &Ui, width: f32, hovered: bool) {
        ui.with_item_width(width, || {
            ui.slider_float(
                im_str!("Drag Speed"),
                &mut self.drag_speed,
                0.005,
                0.08,
            ).build();
            drag_speed_tt(ui, hovered);

            ui.slider_float(im_str!("Height"), &mut self.height, 0.0, 1.0)
                .build();
            height_tt(ui, hovered);

            ui.slider_float(
                im_str!("Time Multi"),
                &mut self.time_multi,
                0.0,
                10.0,
            ).build();
            time_multi_tt(ui, hovered);
        });
    }
}

pub struct MouseVariables {
    pub position: [f32; 2],
    pub hover_time: f32,
    pub hovered: bool,
    pub m1_pressed: bool,
    pub viewport_pressed: Option<usize>,
}

impl MouseVariables {
    pub fn new() -> MouseVariables {
        MouseVariables {
            position: [0.0; 2],
            hover_time: 0.25,
            hovered: false,
            m1_pressed: false,
            viewport_pressed: None,
        }
    }

    fn on_viewport(&self, viewports: &[ViewPort]) -> Option<usize> {
        for (i, viewport) in viewports.iter().enumerate() {
            if viewport.rect.contains(self.position) {
                return Some(i);
            }
        }
        None
    }

    pub fn handle_mouse(
        &mut self,
        mouse: &MouseState,
        viewports: &[ViewPort],
        dimensions: (u32, u32),
    ) {
        let [x, y] = *mouse.mouse.position.as_ref();
        let x = x / dimensions.0 as f32;
        let y = 1.0 - y / dimensions.1 as f32;
        self.position = [x, y];
        self.hovered = mouse.hovered >= self.hover_time;

        let m1 = mouse.mouse.pressed.0;
        if !mouse.on_ui {
            if m1 && !self.m1_pressed {
                self.viewport_pressed = self.on_viewport(viewports);
            } else if !m1 && self.m1_pressed {
                self.viewport_pressed = None;
            }
        }
        self.m1_pressed = m1;
    }
}