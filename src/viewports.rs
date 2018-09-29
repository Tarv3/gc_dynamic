use evec::Evec;
use glium::Rect;
use renderer::{camera::PCamera, Mat4};
use std::iter::Iterator;

#[derive(Copy, Clone, Debug)]
pub enum DivDirection {
    Verticle(f32),
    Horizontal(f32),
}

impl DivDirection {
    fn divide(&self, rect: ViewRect) -> (ViewRect, ViewRect) {
        match self {
            DivDirection::Verticle(r) => {
                let vert_mid = (rect.top + rect.bottom) * 0.5;
                let b_rect = ViewRect {
                    left: rect.left,
                    right: rect.right,
                    bottom: rect.bottom,
                    top: vert_mid,
                };

                let t_rect = ViewRect {
                    left: rect.left,
                    right: rect.right,
                    bottom: vert_mid,
                    top: rect.top,
                };

                (b_rect, t_rect)
            }
            DivDirection::Horizontal(r) => {
                let horiz_mid = (rect.left + rect.right) * 0.5;
                let l_rect = ViewRect {
                    left: rect.left,
                    right: horiz_mid,
                    bottom: rect.bottom,
                    top: rect.top,
                };

                let r_rect = ViewRect {
                    left: horiz_mid,
                    right: rect.right,
                    bottom: rect.bottom,
                    top: rect.top,
                };

                (l_rect, r_rect)
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Division {
    None {
        selected: i32,
        id: usize,
    },
    Ratio {
        id: usize,
        direction: DivDirection,
        a: usize,
        b: usize,
    },
}

impl Division {
    pub fn get_id(&self) -> usize {
        match self {
            Division::None { id, .. } => *id,
            Division::Ratio { id, .. } => *id,
        }
    }
    pub fn is_ratio(&self) -> bool {
        match self {
            Division::None { .. } => false,
            Division::Ratio { .. } => true,
        }
    }

    pub fn build_viewports(
        &self,
        rect: ViewRect,
        divisions: &Vec<Division>,
        viewports: &mut Vec<ViewPort>,
    ) {
        match self {
            Division::None { selected, id } => {
                viewports.push(ViewPort { div_id: *id, rect });
            }
            Division::Ratio {
                direction, a, b, ..
            } => {
                let (rect_a, rect_b) = direction.divide(rect);
                let a_division = divisions[*a];
                let b_division = divisions[*b];
                a_division.build_viewports(rect_a, divisions, viewports);
                b_division.build_viewports(rect_b, divisions, viewports);
            }
        }
    }

    pub fn build_viewports_evec(
        &self,
        rect: ViewRect,
        divisions: &Evec<Division>,
        viewports: &mut Vec<ViewPort>,
    ) {
        match self {
            Division::None { selected, id } => {
                viewports.push(ViewPort { div_id: *id, rect });
            }
            Division::Ratio {
                direction, a, b, ..
            } => {
                let (rect_a, rect_b) = direction.divide(rect);
                let a_division = divisions[*a].expect("Missing division");
                let b_division = divisions[*b].expect("Missing division");
                a_division.build_viewports_evec(rect_a, divisions, viewports);
                b_division.build_viewports_evec(rect_b, divisions, viewports);
            }
        }
    }

    pub fn remove_division(&self, divisions: &mut Evec<Division>) {
        match self {
            Division::Ratio { a, b, id, .. } => {
                let mut selected = 0;
                if let Some(div) = divisions[*a] {
                    div.remove_division(divisions);
                }
                if let Some(div) = divisions[*b] {
                    div.remove_division(divisions);
                }
                divisions.remove(*a);
                divisions.remove(*b);
                divisions[*id] = Some(Division::None { id: *id, selected })
            }
            _ => (),
        }
    }

    pub fn divide(&self, dir: DivDirection, divisions: &mut Evec<Division>) {
        match self {
            Division::None { id, selected } => {
                let a_index = divisions.next_available();
                let a = Division::None {
                    id: a_index,
                    selected: *selected,
                };
                divisions.push(a);
                let b_index = divisions.next_available();
                let b = Division::None {
                    id: b_index,
                    selected: *selected,
                };
                divisions.push(b);
                if let Some(ref mut div) = divisions[*id] {
                    *div = Division::Ratio {
                        id: *id,
                        direction: dir,
                        a: a_index,
                        b: b_index,
                    };
                }
            }
            _ => panic!("Tried to divide an already divided division"),
        }
    }

    pub fn get_selected(&self) -> i32 {
        match self {
            Division::None { selected, .. } => *selected,
            Division::Ratio { .. } => panic!("Tried to get selected from a division"),
        }
    }

    pub fn get_selected_mut(&mut self) -> &mut i32 {
        match self {
            Division::None { selected, .. } => selected,
            Division::Ratio { .. } => panic!("Tried to get selected from a division"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ViewRect {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl ViewRect {
    fn height(&self) -> f32 {
        self.top - self.bottom
    }

    fn width(&self) -> f32 {
        self.right - self.left
    }

    fn from_corners(bl: [f32; 2], tr: [f32; 2]) -> ViewRect {
        ViewRect {
            left: bl[0],
            right: tr[0],
            bottom: bl[1],
            top: tr[1],
        }
    }

    fn aspect_ratio(&self) -> f32 {
        self.width() / self.height()
    }
}

pub struct ViewPort {
    pub div_id: usize,
    pub rect: ViewRect,
}

impl ViewPort {
    pub fn glium_viewport(&self, frame_size: (u32, u32)) -> Rect {
        let horizontal = frame_size.0 as f32;
        let height = frame_size.1 as f32;
        let left = self.rect.left * horizontal;
        let bottom = self.rect.bottom * height;
        let width = self.rect.width() * horizontal;
        let height = self.rect.height() * height;

        Rect {
            left: left.round() as u32,
            bottom: bottom.round() as u32,
            width: width.round() as u32,
            height: height.round() as u32,
        }
    }

    pub fn glium_vp_logicalsize(&self, frame_size: (f64, f64)) -> Rect {
        let horizontal = frame_size.0 as f32;
        let height = frame_size.1 as f32;
        let left = self.rect.left * horizontal;
        let bottom = self.rect.bottom * height;
        let width = self.rect.width() * horizontal;
        let height = self.rect.height() * height;

        Rect {
            left: left.round() as u32,
            bottom: bottom.round() as u32,
            width: width.round() as u32,
            height: height.round() as u32,
        }
    }

    pub fn view_matrix(&self, camera: &PCamera, zoom: f32) -> Mat4 {
        let mut projection = camera.projection;
        let aspect = projection.aspect_ratio();
        projection.set_aspect(aspect * self.rect.aspect_ratio());
        projection.zoomed_matrix(zoom) * camera.look_at_matrix()
    }

    pub fn get_div_selection(&self, divisions: &[Option<Division>]) -> i32 {
        divisions[self.div_id].unwrap().get_selected()
    }
    pub fn get_div_selection_mut<'a>(
        &'a self,
        divisions: &'a mut [Option<Division>],
    ) -> &'a mut i32 {
        divisions[self.div_id].as_mut().unwrap().get_selected_mut()
    }
}