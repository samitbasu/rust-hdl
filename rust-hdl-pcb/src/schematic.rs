use crate::adc::make_ads868x;
use crate::circuit::{CircuitNode, PartDetails, PartInstance, SchematicRotation, Circuit};
use crate::epin::{EPin, EdgeLocation};
use crate::glyph::{estimate_bounding_box, Glyph, Rect, TextJustification};
use svg::node::element::path::Data;
use svg::node::element::Text;
use svg::node::element::{Group, Path};
use svg::Document;
use std::collections::BTreeMap;
use std::fs;
use std::path;

const EM : i32 = 85;
const CHAR_WIDTH : i32 = 55;
const PORT_HALF_HEIGHT: i32 = 55;

// Ugh... Need a proper graphix lib.
fn add_ports(mut doc: Group,
             outline: &[Glyph],
             pins: &BTreeMap<u64, EPin>) -> Group {
    if outline.len() == 0 {
        return doc;
    }
    if let Glyph::OutlineRect(r) = &outline[0] {
        for pin in pins {
            match pin.1.location.edge {
                EdgeLocation::North => {
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
                }
                EdgeLocation::West => {
                    let ax = r.p0.x;
                    let ay = -pin.1.location.offset;
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("x", ax - EM)
                        .set("y", ay)
                        .set("font-family", "monospace")
                        .set("text-anchor", "end")
                        .set("alignment-baseline", "middle")
                        .set("font-size", EM);
                    doc = doc.add(txt);
                    let label_len = pin.1.name.len() as i32 + 2;
                    let data = Data::new()
                        .move_to((ax, ay - PORT_HALF_HEIGHT))
                        .line_to((ax, ay + PORT_HALF_HEIGHT))
                        .line_to((ax - label_len * CHAR_WIDTH, ay + PORT_HALF_HEIGHT))
                        .line_to((ax - label_len * CHAR_WIDTH, ay - PORT_HALF_HEIGHT))
                        .line_to((ax, ay - PORT_HALF_HEIGHT))
                        .close();
                    let path = Path::new()
                        .set("fill", "none")
                        .set("stroke", "black")
                        .set("stroke-width", "10")
                        .set("d", data);
                    doc = doc.add(path);
                }
                EdgeLocation::South => {
                    let ax = pin.1.location.offset;
                    let ay = -r.p0.y;
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("text-anchor", "end")
                        .set("font-family", "monospace")
                        .set("alignment-baseline", "middle")
                        .set(
                            "transform",
                            format!("rotate(-90, {}, {}) translate({} {})", ax, ay + EM, ax, ay + EM),
                        )
                        .set("font-size", EM);
                    doc = doc.add(txt);
                    let label_len = pin.1.name.len() as i32 + 2;
                    let data = Data::new()
                        .move_to((ax - PORT_HALF_HEIGHT, ay))
                        .line_to((ax + PORT_HALF_HEIGHT, ay))
                        .line_to((ax + PORT_HALF_HEIGHT, ay + label_len * CHAR_WIDTH))
                        .line_to((ax - PORT_HALF_HEIGHT, ay + label_len * CHAR_WIDTH))
                        .line_to((ax - PORT_HALF_HEIGHT, ay))
                        .close();
                    let path = Path::new()
                        .set("fill", "none")
                        .set("stroke", "black")
                        .set("stroke-width", "10")
                        .set("d", data);
                    doc = doc.add(path);
                }
                EdgeLocation::East => {
                    let ax = r.p1.x;
                    let ay = -pin.1.location.offset;
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("x", ax + EM)
                        .set("y", ay)
                        .set("font-family", "monospace")
                        .set("text-anchor", "begin")
                        .set("alignment-baseline", "middle")
                        .set("font-size", EM);
                    doc = doc.add(txt);
                    let label_len = pin.1.name.len() as i32 + 2;
                    let data = Data::new()
                        .move_to((ax, ay - PORT_HALF_HEIGHT))
                        .line_to((ax, ay + PORT_HALF_HEIGHT))
                        .line_to((ax + label_len * CHAR_WIDTH, ay + PORT_HALF_HEIGHT))
                        .line_to((ax + label_len * CHAR_WIDTH, ay - PORT_HALF_HEIGHT))
                        .line_to((ax, ay - PORT_HALF_HEIGHT))
                        .close();
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
    doc
}

fn add_pins(mut doc: Group,
            outline: &[Glyph],
            hide_pin_designators: bool,
            pins: &BTreeMap<u64, EPin>) -> Group {
    if outline.len() == 0 {
        return doc;
    }
    if let Glyph::OutlineRect(r) = &outline[0] {
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
                        .line_to((pin.1.location.offset, -r.p1.y - 200));
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
                        .line_to((r.p0.x - 200, -pin.1.location.offset));
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
                        .line_to((pin.1.location.offset, -r.p0.y + 200));
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
                        .line_to((r.p1.x + 200, -pin.1.location.offset));
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
    doc
}

pub fn add_outline_to_path(doc: Group, g: &Glyph) -> Group {
    match g {
        Glyph::OutlineRect(r) => {
            let data = Data::new()
                .move_to((r.p0.x, -r.p0.y))
                .line_to((r.p0.x, -r.p1.y))
                .line_to((r.p1.x, -r.p1.y))
                .line_to((r.p1.x, -r.p0.y))
                .close();
            doc.add(
                Path::new()
                    .set("fill", "#FFFDB0")
                    .set("stroke", "#AE5E46")
                    .set("stroke-width", 5)
                    .set("d", data),
            )
        }
        Glyph::Line(l) => {
            let data = Data::new()
                .move_to((l.p0.x, -l.p0.y))
                .line_to((l.p1.x, -l.p1.y));
            doc.add(
                Path::new()
                    .set("fill", "none")
                    .set("stroke", "#0433FF")
                    .set("stroke-width", 10)
                    .set("d", data),
            )
        }
        Glyph::Pin(p) => match p.location {
            EdgeLocation::East => {
                let data = Data::new()
                    .move_to((p.p0.x, -p.p0.y))
                    .line_to((p.p0.x + p.length, -p.p0.y));
                doc.add(
                    Path::new()
                        .set("fill", "none")
                        .set("stroke", "black")
                        .set("stroke-width", 10)
                        .set("d", data),
                )
            }
            EdgeLocation::West => {
                let data = Data::new()
                    .move_to((p.p0.x, -p.p0.y))
                    .line_to((p.p0.x - p.length, -p.p0.y));
                doc.add(
                    Path::new()
                        .set("fill", "none")
                        .set("stroke", "black")
                        .set("stroke-width", 10)
                        .set("d", data),
                )
            }
            _ => unimplemented!(),
        },
        Glyph::Text(t) => {
            let mut txt = Text::new()
                .add(svg::node::Text::new(&t.text))
                .set("x", t.p0.x)
                .set("y", -t.p0.y)
                .set("font-family", "monospace")
                .set("font-size", EM);
            match t.justify {
                TextJustification::BottomLeft | TextJustification::BottomRight => {
                    txt = txt.set("alignment-baseline", "bottom")
                }
                TextJustification::TopLeft | TextJustification::TopRight => {
                    txt = txt.set("alignment-baseline", "hanging")
                }
                TextJustification::MiddleLeft | TextJustification::MiddleRight => {
                    txt = txt.set("alignment-baseline", "middle")
                }
            }
            match t.justify {
                TextJustification::BottomLeft
                | TextJustification::TopLeft
                | TextJustification::MiddleLeft => txt = txt.set("text-anchor", "begin"),
                TextJustification::BottomRight
                | TextJustification::TopRight
                | TextJustification::MiddleRight => txt = txt.set("text-anchor", "end"),
            }
            doc.add(txt)
        }
        Glyph::Arc(a) => {
            let p1x = a.p0.x as f64 + a.radius * f64::cos(a.start_angle.to_radians());
            let p1y = a.p0.y as f64 + a.radius * f64::sin(a.start_angle.to_radians());
            let p2x = a.p0.x as f64
                + a.radius * f64::cos(a.start_angle.to_radians() + a.sweep_angle.to_radians());
            let p2y = a.p0.y as f64
                + a.radius * f64::sin(a.start_angle.to_radians() + a.sweep_angle.to_radians());
            let data = Data::new()
                .move_to((p1x, p1y))
                .elliptical_arc_to((a.radius, a.radius, 0.0, 0, 1, p2x, p2y));
            doc.add(
                Path::new()
                    .set("fill", "none")
                    .set("stroke", "#0433FF")
                    .set("stroke-width", 10)
                    .set("d", data),
            )
        }
    }
}

pub fn make_flip_lr_part(part: &PartDetails) -> PartDetails {
    let mut fpart = part.clone();
    fpart.outline = part.outline.iter().map(|x| x.fliplr()).collect();
    fpart.pins = part
        .pins
        .iter()
        .map(|x| (*x.0, EPin::new(&x.1.name, x.1.kind, x.1.location.fliplr())))
        .collect();
    fpart
}

pub fn make_flip_ud_part(part: &PartDetails) -> PartDetails {
    let mut fpart = part.clone();
    fpart.outline = part.outline.iter().map(|x| x.flipud()).collect();
    fpart.pins = part
        .pins
        .iter()
        .map(|x| (*x.0, EPin::new(&x.1.name, x.1.kind, x.1.location.flipud())))
        .collect();
    fpart
}

fn get_details_from_instance(x: &PartInstance) -> PartDetails {
    let mut part = match &x.node {
        CircuitNode::Capacitor(c) => &c.details,
        CircuitNode::Resistor(r) => &r.details,
        CircuitNode::Diode(d) => &d.details,
        CircuitNode::Regulator(v) => &v.details,
        CircuitNode::Inductor(l) => &l.details,
        CircuitNode::IntegratedCircuit(u) => &u,
        CircuitNode::Connector(j) => &j,
        CircuitNode::Logic(u) => &u.details,
    }
    .clone();

    if x.schematic_orientation.flipped_lr {
        part = make_flip_lr_part(&part);
    }

    if x.schematic_orientation.flipped_ud {
        part = make_flip_ud_part(&part);
    }

    part
}

impl Into<Group> for &PartInstance {
    fn into(self) -> Group {
        let mut document = Group::new();

        let part = get_details_from_instance(&self);

        for x in &part.outline {
            document = add_outline_to_path(document, x);
        }
        document = add_pins(document,
                            &part.outline,
                            part.hide_pin_designators,
                            &part.pins);

        let r = estimate_bounding_box(&part.outline);
        let data = Data::new()
            .move_to((r.p0.x, -r.p0.y))
            .line_to((r.p0.x, -r.p1.y))
            .line_to((r.p1.x, -r.p1.y))
            .line_to((r.p1.x, -r.p0.y))
            .close();
        document = document.add(
            Path::new()
                .set("fill", "none")
                .set("stroke", "red")
                .set("stroke-width", 5)
                .set("d", data),
        );
        let cx = (r.p0.x + r.p1.x)/2;
        let cy = (r.p0.y + r.p1.y)/2;
        let dx = self.schematic_orientation.center.x;
        let dy = -self.schematic_orientation.center.y;
        dbg!(cx);
        dbg!(cy);
        let transform = format!("{rot} translate({x},{y})",
            x = dx,
            y = dy,
            rot = if self.schematic_orientation.rotation == SchematicRotation::Vertical {
                format!("rotate(-90, {}, {})", dx, dy)
            } else {
                "".to_string()
            }
        );
        document = document.set("transform", transform);
        document
    }
}

pub fn estimate_instance_bounding_box(instance: &PartInstance) -> Rect {
    let part = get_details_from_instance(instance);
    let mut r = estimate_bounding_box(&part.outline);
    if instance.schematic_orientation.rotation == SchematicRotation::Vertical {
        r = r.rot90();
    }
    r
}

pub fn write_circuit_to_svg(circuit: &Circuit, name: &str) {
    let mut top_document = Document::new().set("viewBox", (-2000, -2000, 7000, 7000));
    let mut top: Group = Group::new();
    for instance in &circuit.nodes {
        let part: Group = instance.into();
        top = top.add(part);
    }
    top = add_ports(top, &circuit.outline, &circuit.pins);
    top_document = top_document.add(top);
    svg::save(name, &top_document).unwrap();
}


fn write_to_svg(instance: &PartInstance, name: &str) {
    let r = estimate_instance_bounding_box(instance);
    let mut top_document = Document::new().set("viewBox", (-2000, -2000, 4000, 4000));
    let document: Group = instance.into();
    top_document = top_document.add(document);
    let data = Data::new()
        .move_to((r.p0.x, -r.p0.y))
        .line_to((r.p0.x, -r.p1.y))
        .line_to((r.p1.x, -r.p1.y))
        .line_to((r.p1.x, -r.p0.y))
        .close();
    top_document = top_document.add(
        Path::new()
            .set("fill", "none")
            .set("stroke", "green")
            .set("stroke-width", 2)
            .set("d", data),
    );
    svg::save(name, &top_document).unwrap();
}

// Color for the body of an IC: FFFDB0
// Color for the schematic line: AE5E46

#[test]
fn test_svg_of_part() {
    let u = make_ads868x("ADS8689IPW");
    let i: PartInstance = u.into();
    write_to_svg(&i, "test.svg");
}

pub fn make_svgs(mut part: &mut PartInstance) {
    let details = get_details_from_instance(&part);
    let base = env!("CARGO_MANIFEST_DIR").to_owned()
        + "/symbols/"
        + &details.manufacturer.part_number.replace("/", "_");
    let base_path = std::path::Path::new( &base);
    let base_dir = base_path.parent().unwrap();
    if !base_dir.exists() {
        fs::create_dir_all(base_dir);
    }
    write_to_svg(&part, &format!("{}.svg", base));
    part.schematic_orientation.flipped_lr = true;
    write_to_svg(&part, &format!("{}_lr.svg", base));
    part.schematic_orientation.flipped_lr = false;
    part.schematic_orientation.flipped_ud = true;
    write_to_svg(&part, &format!("{}_ud.svg", base));
    part.schematic_orientation.flipped_lr = true;
    part.schematic_orientation.flipped_ud = true;
    write_to_svg(&part, &format!("{}_lr_ud.svg", base));
    part.schematic_orientation.rotation = SchematicRotation::Vertical;
    part.schematic_orientation.flipped_lr = false;
    part.schematic_orientation.flipped_ud = false;
    write_to_svg(&part, &format!("{}_rot.svg", base));
    part.schematic_orientation.rotation = SchematicRotation::Vertical;
    part.schematic_orientation.flipped_lr = true;
    part.schematic_orientation.flipped_ud = false;
    write_to_svg(&part, &format!("{}_rot_lr.svg", base));
    part.schematic_orientation.rotation = SchematicRotation::Vertical;
    part.schematic_orientation.flipped_lr = false;
    part.schematic_orientation.flipped_ud = true;
    write_to_svg(&part, &format!("{}_rot_ud.svg", base));
    part.schematic_orientation.rotation = SchematicRotation::Vertical;
    part.schematic_orientation.flipped_lr = true;
    part.schematic_orientation.flipped_ud = true;
    write_to_svg(&part, &format!("{}_rot_lr_ud.svg", base));
}
