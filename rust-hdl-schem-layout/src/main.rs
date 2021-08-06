use crate::egui::vec2;
use eframe::egui::{Color32, CtxRef, CursorIcon, PointerButton, Sense, Stroke, Vec2};
use eframe::epi::Frame;
use eframe::{egui, NativeOptions};
use epi::egui::{LayerId, Painter, Pos2, Rect};
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    offset: Vec2,
    zoom: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            offset: (0., 0.).into(),
            zoom: 1.0,
        }
    }
}

impl Transform {
    pub fn translate(self, by: Vec2) -> Self {
        Self {
            offset: self.offset + by,
            zoom: self.zoom,
        }
    }
    pub fn zoom(self, by: f32) -> Self {
        Self {
            offset: self.offset,
            zoom: self.zoom * by,
        }
    }
}

impl std::ops::Mul<[Pos2; 2]> for Transform {
    type Output = [Pos2; 2];

    fn mul(self, rhs: [Pos2; 2]) -> [Pos2; 2] {
        [self * rhs[0], self * rhs[1]]
    }
}

impl std::ops::Mul<Pos2> for Transform {
    type Output = Pos2;

    fn mul(self, rhs: Pos2) -> Pos2 {
        Pos2 {
            x: (rhs.x * self.zoom) + self.offset.x,
            y: (rhs.y * self.zoom) + self.offset.y,
        }
    }
}

impl std::ops::Div<Transform> for Pos2 {
    type Output = Pos2;

    fn div(self, rhs: Transform) -> Pos2 {
        Pos2 {
            x: (self.x - rhs.offset.x) / rhs.zoom,
            y: (self.y - rhs.offset.y) / rhs.zoom,
        }
    }
}

pub struct Component {
    size: Vec2,
    center: Pos2,
    selected: bool,
}

impl Component {
    pub fn new(center: (f32, f32)) -> Component {
        Self {
            center: center.into(),
            ..Default::default()
        }
    }

    pub fn paint(&self, painter: &Painter, transform: Transform) {
        let c: Pos2 = self.center.into();
        let r = (16.0 / 2.0 - 1.0);
        let color = if self.selected {
            Color32::from_rgb(128, 0, 0)
        } else {
            Color32::from_gray(128)
        };
        let stroke = Stroke::new(1.0, color);
        painter.circle_stroke(transform * c, transform.zoom * r, stroke);
        painter.line_segment(transform * [c - vec2(0.0, r), c + vec2(0.0, r)], stroke);
        painter.line_segment(
            transform * [c, c + r * Vec2::angled(TAU * 1.0 / 8.0)],
            stroke,
        );
        painter.line_segment(
            transform * [c, c + r * Vec2::angled(TAU * 3.0 / 8.0)],
            stroke,
        );
    }
}

impl Default for Component {
    fn default() -> Self {
        Self {
            size: (16.0, 16.0).into(),
            center: (100.0, 100.0).into(),
            selected: false,
        }
    }
}

pub struct DemoApp {
    label: String,
    value: i32,
    components: Vec<Component>,
    transform: Transform,
    last_drag_delta: Vec2,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            label: "Hello World".to_owned(),
            value: 0,
            components: vec![
                Component::new((25.0, 25.0)),
                Component::new((100.0, 100.0)),
                Component::new((150.0, 150.0)),
            ],
            transform: Default::default(),
            last_drag_delta: Default::default(),
        }
    }
}

impl epi::App for DemoApp {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.label);
            let size = Vec2::splat(200.0);
            let (response, painter) = ui.allocate_painter(size, Sense::drag());
            let mut canvas_drag = false;
            let right_drag = ui.input().pointer.button_down(PointerButton::Secondary);
            if let Some(hover_pos) = response.hover_pos() {
                let zoom_factor = ui.input().zoom_delta();
                self.transform = self.transform.zoom(zoom_factor);
            }
            if response.drag_started() {
                self.last_drag_delta = (0., 0.).into();
                if right_drag {
                    canvas_drag = true;
                    println!("Secondary drag started");
                } else {
                    let click_pos = response.interact_pointer_pos();
                    if let Some(c) = click_pos {
                        let t = c / self.transform;
                        let z = self.transform * t;
                        println!(
                            "Drag pos {:?} -> {:?} (check {:?})",
                            response.interact_pointer_pos(),
                            t,
                            z
                        );
                        let transform = self.transform;
                        self.components.iter_mut().for_each(|x| {
                            let r = Rect::from_center_size(x.center, x.size);
                            x.selected = r.contains(c / transform);
                        });
                    };
                }
            } else if response.dragged() {
                let inc = response.drag_delta() / self.transform.zoom;
                if right_drag {
                    self.transform = self.transform.translate(inc);
                } else {
                    self.components.iter_mut().for_each(|x| {
                        if x.selected {
                            x.center += inc;
                        }
                    });
                }
            } else {
                self.components.iter_mut().for_each(|x| x.selected = false);
            }
            if canvas_drag || self.components.iter().any(|x| x.selected) {
                ui.output().cursor_icon = CursorIcon::Grab;
            }
            for o in &self.components {
                o.paint(&painter, self.transform);
            }
        });
    }

    fn name(&self) -> &str {
        "Foo"
    }
}

fn main() {
    eframe::run_native(Box::new(DemoApp::default()), NativeOptions::default())
}
