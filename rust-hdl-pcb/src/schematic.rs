use crate::adc::make_ads868x;
use crate::circuit::{
    Circuit, CircuitNode, PartDetails, PartInstance, PartPin, SchematicOrientation,
    SchematicRotation,
};
use crate::epin::{EPin, EdgeLocation};
use crate::glyph::{estimate_bounding_box, Glyph, Rect, TextJustification, Point};
use svg::node::element::path::Data;
use svg::node::element::{Text, Circle};
use svg::node::element::{Group, Path};
use svg::Document;
use std::collections::BTreeMap;
use std::fs;
use std::path;

const EM: i32 = 85;
const PIN_LENGTH: i32 = 200;

fn add_pins(
    mut doc: Group,
    outline: &[Glyph],
    hide_pin_designators: bool,
    pins: &BTreeMap<u64, EPin>,
) -> Group {
    if outline.len() == 0 {
        return doc;
    }
    if let Glyph::OutlineRect(r) = &outline[0] {
        if r.is_empty() {
            return doc;
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
    doc
}

// Adapted from https://stackoverflow.com/questions/21816286/svg-arc-how-to-determine-sweep-and-larg-arc-flags-given-start-end-via-point
fn angle (a: (f64,f64), b: (f64,f64), c: (f64,f64)) -> f64 {
    let pi = std::f64::consts::PI;
    ( f64::atan2(( c.1 - b.1 ) ,( c.0 - b.0 ) )
        - f64::atan2(( a.1 - b.1 ) ,( a.0 - b.0 ) )
        + 3.0 * pi )
        %( 2.0 * pi ) - pi
}

fn find_sweep_flag (start: (f64,f64), via: (f64,f64), end: (f64,f64)) -> i32 {
    return if angle(end, start, via) > 0.0 { 0 } else { 1 };
}


pub fn add_outline_to_path(doc: Group, g: &Glyph, hide_outline: bool) -> Group {
    match g {
        Glyph::OutlineRect(r) => {
            if hide_outline {
                doc
            } else {
                doc.add(
                    Path::new()
                        .set("fill", "#FFFDB0")
                        .set("stroke", "#AE5E46")
                        .set("stroke-width", 5)
                        .set("d", make_rect_into_data(r)),
                )
            }
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
                + a.radius * f64::cos(a.start_angle.to_radians() + a.sweep_angle.to_radians() / 2.0);
            let p2y = a.p0.y as f64
                + a.radius * f64::sin(a.start_angle.to_radians() + a.sweep_angle.to_radians() / 2.0);
            let p3x = a.p0.x as f64
                + a.radius * f64::cos(a.start_angle.to_radians() + a.sweep_angle.to_radians());
            let p3y = a.p0.y as f64
                + a.radius * f64::sin(a.start_angle.to_radians() + a.sweep_angle.to_radians());
            let sweep_flag = find_sweep_flag((p1x, p1y), (p2x, p2y), (p3x , p3y));
            let large_arc_flag = if f64::abs(a.sweep_angle) > 180.0 { 1 } else { 0 };
            let data = Data::new()
                .move_to((p1x, p1y))
                .elliptical_arc_to((a.radius, a.radius, 0.0, large_arc_flag, sweep_flag, p3x, p3y));
            doc.add(
                Path::new()
                    .set("fill", "none")
                    .set("stroke", "#0433FF")
                    .set("stroke-width", 10)
                    .set("d", data),
            )
                /*
                //useful for debugging arcs/ellipses
                .add(Circle::new()
                .set("cx", p1x)
                .set("cy", p1y)
                .set("r", 10)
                .set("fill", "#00FF00")
                .set("stroke", "#00FF00")
                .set("stroke-width", 10))
                .add(Circle::new()
                    .set("cx", p2x)
                    .set("cy", p2y)
                    .set("r", 10)
                    .set("fill", "#FF3399")
                    .set("stroke", "#FF3399")
                    .set("stroke-width", 10))
                .add(Circle::new()
                    .set("cx", p3x)
                    .set("cy", p3y)
                    .set("r", 10)
                    .set("fill", "#FF3300")
                    .set("stroke", "#FF3300")
                    .set("stroke-width", 10))
*/
        },
        Glyph::Circle(a) => {
            doc.add(
                Circle::new()
                    .set("cx", a.p0.x)
                    .set("cy", -a.p0.y)
                    .set("r", a.radius)
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
        CircuitNode::IntegratedCircuit(u) => u,
        CircuitNode::Connector(j) => j,
        CircuitNode::Logic(u) => &u.details,
        CircuitNode::Port(p) => p,
        CircuitNode::Junction(j) => j,
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

fn make_rect_into_data(r: &Rect) -> Data {
    Data::new()
        .move_to((r.p0.x, -r.p0.y))
        .line_to((r.p0.x, -r.p1.y))
        .line_to((r.p1.x, -r.p1.y))
        .line_to((r.p1.x, -r.p0.y))
        .close()
}

fn make_group_from_part_instance(instance: &PartInstance) -> Group {
    let mut document = Group::new();

    let part = get_details_from_instance(instance);

    for x in &part.outline {
        document = add_outline_to_path(document, x, part.hide_part_outline);
    }
    document = add_pins(
        document,
        &part.outline,
        part.hide_pin_designators,
        &part.pins,
    );

    /*
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
    */
    let dx = instance.schematic_orientation.center.x;
    let dy = -instance.schematic_orientation.center.y;
    let transform = format!(
        "{rot} translate({x},{y})",
        x = dx,
        y = dy,
        rot = if instance.schematic_orientation.rotation == SchematicRotation::Vertical {
            format!("rotate(-90, {}, {})", dx, dy)
        } else {
            "".to_string()
        }
    );
    document = document.set("transform", transform);
    document
}

pub fn estimate_instance_bounding_box(instance: &PartInstance) -> Rect {
    let part = get_details_from_instance(instance);
    let mut r = estimate_bounding_box(&part.outline);
    if instance.schematic_orientation.rotation == SchematicRotation::Vertical {
        r = r.rot90();
    }
    r
}

fn map_pin_based_on_orientation(orient: &SchematicOrientation, x: i32, y: i32) -> (i32, i32) {
    let cx = orient.center.x;
    let cy = orient.center.y;
    return match orient.rotation {
        SchematicRotation::Horizontal => (x + cx, -(y + cy)),
        SchematicRotation::Vertical => (-y + cx, -(x + cy)),
    };
}

fn map_pin_based_on_outline_and_orientation(
    pin: &EPin,
    r: &Rect,
    orientation: &SchematicOrientation,
    len: i32,
) -> (i32, i32) {
    return match &pin.location.edge {
        EdgeLocation::North => {
            map_pin_based_on_orientation(&orientation, pin.location.offset, r.p1.y + len)
        }
        EdgeLocation::West => {
            map_pin_based_on_orientation(&orientation, r.p0.x - len, pin.location.offset)
        }
        EdgeLocation::East => {
            map_pin_based_on_orientation(&orientation, r.p1.x + len, pin.location.offset)
        }
        EdgeLocation::South => {
            map_pin_based_on_orientation(&orientation, pin.location.offset, r.p0.y - len)
        }
    };
}

fn get_pin_net_location(circuit: &Circuit, pin: &PartPin) -> (i32, i32) {
    for instance in &circuit.nodes {
        if instance.id == pin.part_id {
            let part = get_details_from_instance(instance);
            let pin = &part.pins[&pin.pin];
            return if let Glyph::OutlineRect(r) = &part.outline[0] {
                map_pin_based_on_outline_and_orientation(
                    pin,
                    r,
                    &instance.schematic_orientation,
                    PIN_LENGTH,
                )
            } else {
                // Parts without an outline rect are just virtual...
                (instance.schematic_orientation.center.x,
                 -instance.schematic_orientation.center.y)
            }
        }
    }
    panic!("No pin found!")
}

pub fn write_circuit_to_svg(circuit: &Circuit, name: &str) {
    let mut top_document = Document::new().set("viewBox", (-2000, -2000, 7000, 7000));
    let mut top: Group = Group::new();
    for instance in &circuit.nodes {
        let part = make_group_from_part_instance(instance);
        top = top.add(part);
    }
    //    top = add_ports(top, &circuit.outline, &circuit.pins);
    top_document = top_document.add(top);
    // Draw the nets
    for net in &circuit.nets {
        for wire in &net.logical_wires {
            let start = get_pin_net_location(&circuit, &wire.start);
            let end = get_pin_net_location(&circuit, &wire.end);
            let mut data = Data::new().move_to(start);
            for pos in &wire.waypoints {
                data = data.line_to((pos.0, -pos.1))
            }
            data = data.line_to(end);
            let path = Path::new()
                .set("fill", "none")
                .set("stroke", "#000080")
                .set("stroke-width", "10")
                .set("d", data);
            top_document = top_document.add(path);
        }
    }
    svg::save(name, &top_document).unwrap();
}

fn write_to_svg(instance: &PartInstance, name: &str) {
    let r = estimate_instance_bounding_box(instance);
    let mut top_document = Document::new().set("viewBox", (-2000, -2000, 4000, 4000));
    let document = make_group_from_part_instance(instance);
    top_document = top_document.add(document);
    top_document = top_document.add(
        Path::new()
            .set("fill", "none")
            .set("stroke", "green")
            .set("stroke-width", 2)
            .set("d", make_rect_into_data(&r)),
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
