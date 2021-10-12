use std::collections::BTreeMap;
use std::fs;

use svg::node::element::path::Data;
use svg::node::element::{Circle, Text};
use svg::node::element::{Group, Path};
use svg::Document;

use rust_hdl_pcb_core::prelude::*;

const EM: i32 = 85;

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
fn angle(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
    let pi = std::f64::consts::PI;
    (f64::atan2(c.1 - b.1, c.0 - b.0) - f64::atan2(a.1 - b.1, a.0 - b.0) + 3.0 * pi) % (2.0 * pi)
        - pi
}

fn find_sweep_flag(start: (f64, f64), via: (f64, f64), end: (f64, f64)) -> i32 {
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
                + a.radius
                    * f64::cos(a.start_angle.to_radians() + a.sweep_angle.to_radians() / 2.0);
            let p2y = a.p0.y as f64
                + a.radius
                    * f64::sin(a.start_angle.to_radians() + a.sweep_angle.to_radians() / 2.0);
            let p3x = a.p0.x as f64
                + a.radius * f64::cos(a.start_angle.to_radians() + a.sweep_angle.to_radians());
            let p3y = a.p0.y as f64
                + a.radius * f64::sin(a.start_angle.to_radians() + a.sweep_angle.to_radians());
            let sweep_flag = find_sweep_flag((p1x, p1y), (p2x, p2y), (p3x, p3y));
            let large_arc_flag = if f64::abs(a.sweep_angle) > 180.0 {
                1
            } else {
                0
            };
            let data = Data::new().move_to((p1x, p1y)).elliptical_arc_to((
                a.radius,
                a.radius,
                0.0,
                large_arc_flag,
                sweep_flag,
                p3x,
                p3y,
            ));
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
        }
        Glyph::Circle(a) => doc.add(
            Circle::new()
                .set("cx", a.p0.x)
                .set("cy", -a.p0.y)
                .set("r", a.radius),
        ),
    }
}

fn make_rect_into_data(r: &Rect) -> Data {
    Data::new()
        .move_to((r.p0.x, -r.p0.y))
        .line_to((r.p0.x, -r.p1.y))
        .line_to((r.p1.x, -r.p1.y))
        .line_to((r.p1.x, -r.p0.y))
        .close()
}

fn make_group_from_part_instance(instance: &PartInstance, layout: &SchematicLayout) -> Group {
    let mut document = Group::new();

    let part = get_details_from_instance(instance, layout);

    for x in &part.outline {
        document = add_outline_to_path(document, x, part.hide_part_outline);
    }
    document = add_pins(
        document,
        &part.outline,
        part.hide_pin_designators,
        &part.pins,
    );

    let schematic_orientation = layout.part(&instance.id);
    let dx = schematic_orientation.center.0;
    let dy = -schematic_orientation.center.1;
    let transform = format!(
        "{rot} translate({x},{y})",
        x = dx,
        y = dy,
        rot = if schematic_orientation.rotation == SchematicRotation::Vertical {
            format!("rotate(-90, {}, {})", dx, dy)
        } else {
            "".to_string()
        }
    );
    document = document.set("transform", transform);
    document
}

pub fn estimate_instance_bounding_box(instance: &PartInstance, layout: &SchematicLayout) -> Rect {
    let part = get_details_from_instance(instance, layout);
    let mut r = estimate_bounding_box(&part.outline);
    let schematic_orientation = layout.part(&instance.id);
    if schematic_orientation.rotation == SchematicRotation::Vertical {
        r = r.rot90();
    }
    r
}

pub fn write_circuit_to_svg(circuit: &Circuit, layout: &SchematicLayout, name: &str) {
    let mut top_document = Document::new().set("viewBox", (-2000, -2000, 7000, 7000));
    let mut top: Group = Group::new();
    for instance in &circuit.nodes {
        let part = make_group_from_part_instance(instance, layout);
        top = top.add(part);
    }
    //    top = add_ports(top, &circuit.outline, &circuit.pins);
    top_document = top_document.add(top);
    // Draw the nets
    for net in &circuit.nets {
        // Build the port definitions
        let ports = net
            .pins
            .iter()
            .map(|x| get_pin_net_location(&circuit, layout, x))
            .collect::<Vec<_>>();
        // Now walk the layout
        let mut net_layout = layout.net(&net.name);
        if net_layout.len() == 0 {
            net_layout = make_rat_layout(ports.len());
        }
        let mut data = Data::new();
        let mut lp = (0, 0);
        for cmd in net_layout {
            match cmd {
                NetLayoutCmd::MoveToPort(n) => {
                    data = data.move_to(ports[n - 1]);
                    lp = ports[n - 1];
                }
                NetLayoutCmd::LineToPort(n) => {
                    data = data.line_to(ports[n - 1]);
                    lp = ports[n - 1];
                }
                NetLayoutCmd::MoveToCoords(x, y) => {
                    data = data.move_to((x, -y));
                    lp = (x, -y);
                }
                NetLayoutCmd::LineToCoords(x, y) => {
                    data = data.line_to((x, -y));
                    lp = (x, -y);
                }
                NetLayoutCmd::Junction => {
                    top_document = top_document.add(
                        Circle::new()
                            .set("cx", lp.0)
                            .set("cy", lp.1)
                            .set("r", 25)
                            .set("fill", "black"),
                    );
                }
            }
        }
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "#000080")
            .set("stroke-width", "10")
            .set("d", data);
        top_document = top_document.add(path);
    }
    svg::save(name, &top_document).unwrap();
}

fn write_to_svg(instance: &PartInstance, layout: &SchematicLayout, name: &str) {
    let r = estimate_instance_bounding_box(instance, &layout);
    let mut top_document = Document::new().set("viewBox", (-2000, -2000, 4000, 4000));
    let document = make_group_from_part_instance(instance, &layout);
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

#[cfg(test)]
pub fn make_ads868x(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("ADS868"));
    assert!(part_number.ends_with("IPW"));
    let pins = vec![
        pin!("DGND", PowerReturn, 300, South),
        pin!("AVDD", PowerSink, -200, North),
        pin!("AGND", PowerReturn, -200, South),
        pin!("REFIO", Passive, 0, West),
        pin!("REFGND", PowerReturn, -800, West),
        pin!("REFCAP", Passive, -300, West),
        pin!("AIN_P", Passive, 800, West),
        pin!("AIN_GND", Passive, 400, West),
        pin!("~RST", InputInverted, -900, East),
        pin!("SDI", Input, -700, East),
        pin!("CONVST/~CS", InputInverted, -500, East),
        pin!("SCLK", Input, -300, East),
        pin!("SDO-0", Output, -100, East),
        pin!("ALARM/SDO-1/GPO", Output, 400, East),
        pin!("RVS", Output, 700, East),
        pin!("DVDD", PowerSink, 300, North),
    ];
    CircuitNode::IntegratedCircuit(PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer {
            name: "TI".to_string(),
            part_number: part_number.into(),
        },
        description: "16-bit high-speed single supply SAR ADC".to_string(),
        comment: "".to_string(),
        hide_pin_designators: false,
        hide_part_outline: false,
        pins: pin_list(pins),
        outline: vec![
            make_ic_body(-800, -1400, 900, 1200),
            make_label(-800, 1200, "U?", TextJustification::BottomLeft),
            make_label(-800, -1400, part_number, TextJustification::TopLeft),
        ],
        size: SizeCode::TSSOP(16),
    })
}

#[test]
fn test_svg_of_part() {
    let u = make_ads868x("ADS8689IPW");
    let i: PartInstance = instance(u, "p1");
    let l = SchematicLayout::default();
    write_to_svg(&i, &l, "test.svg");
}

pub fn make_svgs(mut part: &mut PartInstance) {
    let mut layout = SchematicLayout::default();
    let details = get_details_from_instance(&part, &layout);
    let base = env!("CARGO_MANIFEST_DIR").to_owned()
        + "/symbols/"
        + &details.manufacturer.part_number.replace("/", "_");
    let base_path = std::path::Path::new(&base);
    let base_dir = base_path.parent().unwrap();
    if !base_dir.exists() {
        fs::create_dir_all(base_dir).expect("failed to create symbols directory");
    }
    write_to_svg(&part, &layout, &format!("{}.svg", base));
    layout.set_part(
        &part.id,
        SchematicOrientation {
            flipped_lr: true,
            ..Default::default()
        },
    );
    write_to_svg(&part, &layout, &format!("{}_lr.svg", base));
    layout.set_part(
        &part.id,
        SchematicOrientation {
            flipped_ud: true,
            ..Default::default()
        },
    );
    write_to_svg(&part, &layout, &format!("{}_ud.svg", base));
    layout.set_part(
        &part.id,
        SchematicOrientation {
            flipped_lr: true,
            flipped_ud: true,
            ..Default::default()
        },
    );
    write_to_svg(&part, &layout, &format!("{}_lr_ud.svg", base));
    layout.set_part(
        &part.id,
        SchematicOrientation {
            rotation: SchematicRotation::Vertical,
            ..Default::default()
        },
    );
    write_to_svg(&part, &layout, &format!("{}_rot.svg", base));
    layout.set_part(
        &part.id,
        SchematicOrientation {
            rotation: SchematicRotation::Vertical,
            flipped_lr: true,
            ..Default::default()
        },
    );
    write_to_svg(&part, &layout, &format!("{}_rot_lr.svg", base));
    layout.set_part(
        &part.id,
        SchematicOrientation {
            rotation: SchematicRotation::Vertical,
            flipped_ud: true,
            ..Default::default()
        },
    );
    write_to_svg(&part, &layout, &format!("{}_rot_ud.svg", base));
    layout.set_part(
        &part.id,
        SchematicOrientation {
            rotation: SchematicRotation::Vertical,
            flipped_lr: true,
            flipped_ud: true,
            ..Default::default()
        },
    );
    write_to_svg(&part, &layout, &format!("{}_rot_lr_ud.svg", base));
}
