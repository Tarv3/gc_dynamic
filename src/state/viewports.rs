use evec::Evec;
use glium::Rect;
use renderer::{camera::PCamera, Mat4};

#[derive(Copy, Clone, Debug)]
pub enum DivDirection {
    Verticle(f32),
    Horizontal(f32),
}

impl DivDirection {
    fn divide(&self, rect: ViewRect) -> (ViewRect, ViewRect) {
        match self {
            DivDirection::Verticle(r) => {
                let vert_mid = (rect.top + rect.bottom) * r;
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
                let horiz_mid = (rect.left + rect.right) * r;
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
        parent: Option<usize>,
        id: usize,
    },
    Ratio {
        parent: Option<usize>,
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
    pub fn get_parent(&self) -> Option<usize> {
        match self {
            Division::None { parent, .. } => *parent,
            Division::Ratio { parent, .. } => *parent,
        }
    }
    pub fn build_viewports(
        &self,
        rect: ViewRect,
        divisions: &Vec<Division>,
        viewports: &mut Vec<ViewPort>,
    ) {
        match self {
            Division::None { id, .. } => {
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
            Division::None { id, .. } => {
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

    pub fn remove_division(&self, divisions: &mut Evec<Division>, selected: i32) {
        match self {
            Division::Ratio { id, parent, .. } => {
                self.remove_children(divisions);
                divisions[*id] = Some(Division::None {
                    parent: *parent,
                    id: *id,
                    selected,
                })
            }
            _ => panic!("Tried to remove a none division"),
        }
    }

    fn remove_children(&self, divisions: &mut Evec<Division>) {
        match self {
            Division::Ratio { a, b, .. } => {
                if let Some(div) = divisions[*a] {
                    div.remove_children(divisions);
                }
                if let Some(div) = divisions[*b] {
                    div.remove_children(divisions);
                }
                divisions.remove(*a);
                divisions.remove(*b);
            }
            _ => (),
        }
    }

    pub fn divide(&self, dir: DivDirection, divisions: &mut Evec<Division>) {
        match self {
            Division::None { id, selected, .. } => {
                let a_index = divisions.next_available();
                let a = Division::None {
                    parent: Some(*id),
                    id: a_index,
                    selected: *selected,
                };
                divisions.push(a);
                let b_index = divisions.next_available();
                let b = Division::None {
                    parent: Some(*id),
                    id: b_index,
                    selected: *selected,
                };
                divisions.push(b);
                if let Some(ref mut div) = divisions[*id] {
                    *div = Division::Ratio {
                        parent: div.get_parent(),
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

    pub fn get_selected(&self) -> Option<i32> {
        match self {
            Division::None { selected, .. } => Some(*selected),
            Division::Ratio { .. } => None,
        }
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut i32> {
        match self {
            Division::None { selected, .. } => Some(selected),
            Division::Ratio { .. } => None,
        }
    }

    pub fn get_children(&self) -> (usize, usize) {
        match self {
            Division::Ratio { a, b, .. } => (*a, *b),
            _ => panic!("Tried to get children from none"),
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

    pub fn contains(&self, pos: [f32; 2]) -> bool {
        let x = pos[0];
        let y = pos[1];

        x < self.right && x > self.left && y < self.top && y > self.bottom
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

    pub fn get_div_selection(&self, divisions: &[Option<Division>]) -> Option<i32> {
        let value = divisions[self.div_id];
        match value {
            Some(div) => div.get_selected(),
            None => None,
        }
    }

    pub fn get_div_selection_mut<'a>(
        &'a self,
        divisions: &'a mut [Option<Division>],
    ) -> Option<&'a mut i32> {
        let value = divisions[self.div_id].as_mut();
        match value {
            Some(div) => div.get_selected_mut(),
            None => None,
        }
    }

    pub fn can_render(&self) -> bool {
        self.rect.left < self.rect.right && self.rect.bottom < self.rect.top
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VPSettings {
    pub menu_open: bool,
    pub show_range: bool,
    pub cam: Option<PCamera>,
}
