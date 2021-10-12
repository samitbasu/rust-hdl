use crate::egui::vec2;
use eframe::egui::{Color32, CtxRef, CursorIcon, PointerButton, Sense, Stroke, Vec2, Shape};
use eframe::epi::Frame;
use eframe::{egui, NativeOptions};
use epi::egui::{LayerId, Painter, Pos2, Rect, Rgba};
use std::f32::consts::TAU;
use std::fs::File;
use std::path::PathBuf;
use rust_hdl_pcb::circuit::Circuit;
use rust_hdl_pcb::schematic_layout::{SchematicLayout, SchematicRotation};
use rust_hdl_pcb::schematic::{get_details_from_instance, estimate_instance_bounding_box};
use rust_hdl_pcb::glyph::Glyph;
use rust_hdl_pcb::glyph;
use std::collections::BTreeMap;
use rust_hdl_pcb::epin::{EPin, EdgeLocation};

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

pub struct CircuitPainter {
    pub painter: Painter,
    pub transform: Transform,
    pub offset: glyph::Point,
    pub rot90: bool,
}


impl CircuitPainter {
    pub fn new(painter: Painter, transform: Transform) -> CircuitPainter {
        CircuitPainter {
            painter,
            transform,
            offset: glyph::Point::zero(),
            rot90: false,
        }
    }
    // 90 degree rotation:
    // tx = x cos 90 - y sin 90
    // ty = x sin 90 + y cos 90
    pub fn map_pt(&self, pt: glyph::Point) -> Pos2 {
        let mut pt = pt;
        if self.rot90 {
            pt = (-pt.y, pt.x).into()
        }
        let mut pt = pt + self.offset;
        let mut pt = Pos2::new(pt.x as f32, pt.y as f32);
        pt = self.transform * pt;
        pt
    }
    pub fn map_rect(&self, rect: glyph::Rect) -> Rect {
        let p0 = self.map_pt(rect.p0);
        let p1 = self.map_pt(rect.p1);
        Rect::from_two_pos(p0, p1)
    }
    pub fn render(&mut self, circuit: &Circuit, layout: &SchematicLayout) {
        for instance in &circuit.nodes {
            let part = get_details_from_instance(instance, layout);
            let orientation = layout.part(&instance.id);
            self.offset = orientation.center.into();
            if orientation.rotation == SchematicRotation::Vertical {
                self.rot90 = true;
            } else {
                self.rot90 = false;
            }
            for x in &part.outline {
                self.add_outline(x, part.hide_part_outline);
            }
        }
    }
    pub fn add_outline(&mut self, glyph: &Glyph, hide_outline: bool) {
        match &glyph {
            Glyph::OutlineRect(r) => {
                if !hide_outline {
                    self.painter.add(Shape::Rect {
                        rect: self.map_rect(*r),
                        corner_radius: 0.0,
                        fill: Color32::from_rgb(0xFF, 0xFD, 0xB0),
                        stroke: Stroke::new(1.0, Color32::from_rgb(0xAE, 0x5E, 0x46))
                    });
                }
            }
            Glyph::Line(l) => {
                self.painter.add(Shape::LineSegment {
                    points: [self.map_pt(l.p0), self.map_pt(l.p1)],
                    stroke: Stroke::new(1.0, Color32::from_rgb(0x04, 0x33, 0xff)),
                });
            }
            _ => {}
        }
    }
    fn add_pins(&mut self, outline: &[glyph::Glyph], hide_pin_designators: bool, pins: &BTreeMap<u64, EPin>) {
        if outline.len() == 0 {
            return;
        }
        if let Glyph::OutlineRect(r) = &outline[0] {
            if r.is_empty() {
                return;
            }
            for pin in pins {
                match pin.1.location.edge {
                    EdgeLocation::North => {
                        if !hide_pin_designators {
                            let pn_x = pin.1.location.offset;
                            let pn_y = -(r.p0.y.max(r.p1.y));
                            let txt = Text::new()
                                .add(svg::node::Text::new(format!("{}", pin.0)))
                                .set("x", 0)
                                .set("y", 0)
                                .set("text-anchor", "start")
                                .set("font-family", "monospace")
                                .set("alignment-baseline", "bottom")
                                .set(
                                    "transform",
                                    format!(
                                        "rotate(-90, {}, {}) translate({} {})",
                                        pn_x,
                                        pn_y,
                                        pn_x + 50,
                                        pn_y - 15
                                    ),
                                )
                                .set("font-size", EM);
                            doc = doc.add(txt)
                        }
                        let tx = pin.1.location.offset;
                        let ty = -r.p1.y + EM;
                        let txt = Text::new()
                            .add(svg::node::Text::new(&pin.1.name))
                            .set("text-anchor", "end")
                            .set("font-family", "monospace")
                            .set("alignment-baseline", "middle")
                            .set(
                                "transform",
                                format!("rotate(-90, {}, {}) translate({} {})", tx, ty, tx, ty),
                            )
                            .set("font-size", EM);
                        doc = doc.add(txt);
                        let data = Data::new()
                            .move_to((pin.1.location.offset, -r.p1.y))
                            .line_to((pin.1.location.offset, -r.p1.y - PIN_LENGTH));
                        let path = Path::new()
                            .set("fill", "none")
                            .set("stroke", "black")
                            .set("stroke-width", "10")
                            .set("d", data);
                        doc = doc.add(path);
                    }
                    EdgeLocation::West => {
                        if !hide_pin_designators {
                            let txt = Text::new()
                                .add(svg::node::Text::new(format!("{}", pin.0)))
                                .set("x", r.p0.x - EM)
                                .set("y", -pin.1.location.offset - 15)
                                .set("font-family", "monospace")
                                .set("text-anchor", "end")
                                .set("font-size", EM);
                            doc = doc.add(txt);
                        }
                        let txt = Text::new()
                            .add(svg::node::Text::new(&pin.1.name))
                            .set("x", r.p0.x + EM)
                            .set("y", -pin.1.location.offset)
                            .set("font-family", "monospace")
                            .set("text-anchor", "begin")
                            .set("alignment-baseline", "middle")
                            .set("font-size", EM);
                        doc = doc.add(txt);
                        let data = Data::new()
                            .move_to((r.p0.x, -pin.1.location.offset))
                            .line_to((r.p0.x - PIN_LENGTH, -pin.1.location.offset));
                        let path = Path::new()
                            .set("fill", "none")
                            .set("stroke", "black")
                            .set("stroke-width", "10")
                            .set("d", data);
                        doc = doc.add(path);
                    }
                    EdgeLocation::South => {
                        if !hide_pin_designators {
                            let pn_x = pin.1.location.offset;
                            let pn_y = -(r.p0.y.min(r.p1.y));
                            let txt = Text::new()
                                .add(svg::node::Text::new(format!("{}", pin.0)))
                                .set("x", 0)
                                .set("y", 0)
                                .set("font-family", "monospace")
                                .set("text-anchor", "end")
                                .set("alignment-baseline", "bottom")
                                .set(
                                    "transform",
                                    format!(
                                        "rotate(-90, {}, {}) translate({} {})",
                                        pn_x,
                                        pn_y,
                                        pn_x - EM,
                                        pn_y - 15
                                    ),
                                )
                                .set("font-size", EM);
                            doc = doc.add(txt)
                        }
                        let tx = pin.1.location.offset;
                        let ty = -r.p0.y - EM;
                        let txt = Text::new()
                            .add(svg::node::Text::new(&pin.1.name))
                            .set("text-anchor", "begin")
                            .set("font-family", "monospace")
                            .set("alignment-baseline", "middle")
                            .set(
                                "transform",
                                format!("rotate(-90, {}, {}) translate({} {})", tx, ty, tx, ty),
                            )
                            .set("font-size", EM);
                        doc = doc.add(txt);
                        let data = Data::new()
                            .move_to((pin.1.location.offset, -r.p0.y))
                            .line_to((pin.1.location.offset, -r.p0.y + PIN_LENGTH));
                        let path = Path::new()
                            .set("fill", "none")
                            .set("stroke", "black")
                            .set("stroke-width", "10")
                            .set("d", data);
                        doc = doc.add(path);
                    }
                    EdgeLocation::East => {
                        if !hide_pin_designators {
                            let txt = Text::new()
                                .add(svg::node::Text::new(format!("{}", pin.0)))
                                .set("x", r.p1.x + EM)
                                .set("y", -pin.1.location.offset - 15)
                                .set("font-family", "monospace")
                                .set("text-anchor", "begin")
                                .set("font-size", EM);
                            doc = doc.add(txt);
                        }
                        let txt = Text::new()
                            .add(svg::node::Text::new(&pin.1.name))
                            .set("x", r.p1.x - EM)
                            .set("y", -pin.1.location.offset)
                            .set("font-family", "monospace")
                            .set("text-anchor", "end")
                            .set("alignment-baseline", "middle")
                            .set("font-size", EM);
                        doc = doc.add(txt);
                        let data = Data::new()
                            .move_to((r.p1.x, -pin.1.location.offset))
                            .line_to((r.p1.x + PIN_LENGTH, -pin.1.location.offset));
                        let path = Path::new()
                            .set("fill", "none")
                            .set("stroke", "black")
                            .set("stroke-width", "10")
                            .set("d", data);
                        doc = doc.add(path);
                    }
                }
            }
        }
    }
}


pub struct DemoApp {
    label: String,
    value: i32,
    circuit: Circuit,
    layout: SchematicLayout,
    components: Vec<Component>,
    transform: Transform,
    last_drag_delta: Vec2,
}

/*
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
*/

impl epi::App for DemoApp {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.label);
            let size = Vec2::splat(800.0);
            let (response, painter) = ui.allocate_painter(size, Sense::drag());
            painter.add(Shape::Rect {
                rect: painter.clip_rect(),
                corner_radius: 0.0,
                fill: Color32::from_rgb(0xFF_u8, 0xFC_u8, 0xF8_u8),
                stroke: Default::default()
            });
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
                    self.transform = self.transform.translate(response.drag_delta());
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
            let mut tp = CircuitPainter::new(painter, self.transform);
            tp.render(&self.circuit, &self.layout);

//            for o in &self.components {
//                o.paint(&painter, self.transform);
//            }
        });
    }

    fn name(&self) -> &str {
        "Foo"
    }
}

fn main() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut file = File::open(path.join("digital_3v3_supply_circuit.json")).unwrap();
    let circuit : Circuit = serde_json::from_reader(&mut file).unwrap();
    let mut layout = File::open(path.join("digital_3v3_supply_layout.json")).unwrap();
    let layout : SchematicLayout = serde_json::from_reader(&mut layout).unwrap();
    let comps = circuit.nodes
        .iter()
        .map(|component|
            {
                let rect = estimate_instance_bounding_box(&component, &layout);
                Component {
                    size: ((rect.p1.x - rect.p0.x) as f32, (rect.p1.y - rect.p0.y) as f32).into(),
                    center: (((rect.p1.x + rect.p0.x) / 2) as f32, ((rect.p1.y + rect.p0.y) / 2) as f32).into(),
                    selected: false
                }
            }).collect::<Vec<_>>();
    let app = DemoApp {
        label: "Test".into(),
        value: 0,
        components: comps,
        circuit,
        layout,
        transform: Transform{
            offset: Default::default(),
            zoom: 160.0/500.0,
        },
        last_drag_delta: Default::default()
    };
    eframe::run_native(Box::new(app), NativeOptions::default())
}
