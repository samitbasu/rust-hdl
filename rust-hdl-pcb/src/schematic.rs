use crate::adc::make_ads868x;
use crate::circuit::{CircuitNode, PartDetails, SchematicRotation};
use crate::epin::{EPin, EdgeLocation};
use crate::glyph::{Glyph, Rect, estimate_bounding_box, TextJustification};
use crate::isolators::make_iso7741edwrq1;
use svg::node::element::path::Data;
use svg::node::element::{Path, Group};
use svg::node::element::Text;
use svg::Document;

pub fn add_pins(mut doc: Group, part: &PartDetails) -> Group {
    if part.outline.len() == 0 {
        return doc;
    }
    if let Glyph::OutlineRect(r) = &part.outline[0] {
        for pin in &part.pins {
            match pin.1.location.edge {
                EdgeLocation::North => {
                    if !part.hide_pin_designators {
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
                            .set("font-size", 85);
                        doc = doc.add(txt)
                    }
                    let tx = pin.1.location.offset;
                    let ty = -r.p1.y + 85;
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("text-anchor", "end")
                        .set("font-family", "monospace")
                        .set("alignment-baseline", "middle")
                        .set(
                            "transform",
                            format!("rotate(-90, {}, {}) translate({} {})", tx, ty, tx, ty),
                        )
                        .set("font-size", 85);
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
                    if !part.hide_pin_designators {
                        let txt = Text::new()
                            .add(svg::node::Text::new(format!("{}", pin.0)))
                            .set("x", r.p0.x - 85)
                            .set("y", -pin.1.location.offset - 15)
                            .set("font-family", "monospace")
                            .set("text-anchor", "end")
                            .set("font-size", 85);
                        doc = doc.add(txt);
                    }
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("x", r.p0.x + 85)
                        .set("y", -pin.1.location.offset)
                        .set("font-family", "monospace")
                        .set("text-anchor", "begin")
                        .set("alignment-baseline", "middle")
                        .set("font-size", 85);
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
                    if !part.hide_pin_designators {
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
                                    pn_x - 85,
                                    pn_y - 15
                                ),
                            )
                            .set("font-size", 85);
                        doc = doc.add(txt)
                    }
                    let tx = pin.1.location.offset;
                    let ty = -r.p0.y - 85;
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("text-anchor", "begin")
                        .set("font-family", "monospace")
                        .set("alignment-baseline", "middle")
                        .set(
                            "transform",
                            format!("rotate(-90, {}, {}) translate({} {})", tx, ty, tx, ty),
                        )
                        .set("font-size", 85);
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
                    if !part.hide_pin_designators {
                        let txt = Text::new()
                            .add(svg::node::Text::new(format!("{}", pin.0)))
                            .set("x", r.p1.x + 85)
                            .set("y", -pin.1.location.offset - 15)
                            .set("font-family", "monospace")
                            .set("text-anchor", "begin")
                            .set("font-size", 85);
                        doc = doc.add(txt);
                    }
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("x", r.p1.x - 85)
                        .set("y", -pin.1.location.offset)
                        .set("font-family", "monospace")
                        .set("text-anchor", "end")
                        .set("alignment-baseline", "middle")
                        .set("font-size", 85);
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
                .set("font-size", 85);
            match t.justify {
                TextJustification::BottomLeft |
                TextJustification::BottomRight => {
                    txt = txt.set("alignment-baseline", "bottom")
                }
                TextJustification::TopLeft |
                TextJustification::TopRight => {
                    txt = txt.set("alignment-baseline", "hanging")
                }
                TextJustification::MiddleLeft |
                TextJustification::MiddleRight => {
                    txt = txt.set("alignment-baseline", "middle")
                }
            }
            match t.justify {
                TextJustification::BottomLeft |
                TextJustification::TopLeft |
                TextJustification::MiddleLeft => {
                    txt = txt.set("text-anchor", "begin")
                }
                TextJustification::BottomRight |
                TextJustification::TopRight |
                TextJustification::MiddleRight => {
                    txt = txt.set("text-anchor", "end")
                }
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
    fpart.pins = part.pins.iter().map(|x|
        (*x.0, EPin::new(&x.1.name, x.1.kind, x.1.location.fliplr()))
    ).collect();
    fpart
}

pub fn make_flip_ud_part(part: &PartDetails) -> PartDetails {
    let mut fpart = part.clone();
    fpart.outline = part.outline.iter().map(|x| x.flipud()).collect();
    fpart.pins = part.pins.iter().map(|x|
        (*x.0, EPin::new(&x.1.name, x.1.kind, x.1.location.flipud())))
        .collect();
    fpart
}

impl Into<Group> for &PartDetails {
    fn into(self) -> Group {
        let part = self;
        let mut document = Group::new();

        for x in &part.outline {
            document = add_outline_to_path(document, x);
        }
        document = add_pins(document, &part);

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
        if part.schematic_orientation.rotation == SchematicRotation::Vertical {
            document = document.set("transform", "rotate(-90)");
        }
        document
    }
}


fn write_to_svg(part: &PartDetails, name: &str) {
    let mut top_document = Document::new().set("viewBox", (-2000, -2000, 4000, 4000));
    let document: Group = part.into();
    top_document = top_document.add(document);
    svg::save(name, &top_document).unwrap();
}

// Color for the body of an IC: FFFDB0
// Color for the schematic line: AE5E46
pub fn make_svg(part: &PartDetails) {
    write_to_svg(part, &format!("{}.svg", part.manufacturer.part_number.replace("/", "_")))
}

#[test]
fn test_svg_of_part() {
    let u = match make_ads868x("ADS8689IPW") {
        CircuitNode::IntegratedCircuit(u) => u,
        _ => panic!("Unexpected node"),
    };
    make_svg(&u);
}

pub fn make_svgs(part: &PartDetails) {
    let base = part.manufacturer.part_number.replace("/", "_");
    write_to_svg(part, &format!("{}.svg", base));
    write_to_svg(&make_flip_lr_part(part), &format!("{}_lr.svg", base));
    write_to_svg(&make_flip_ud_part(part), &format!("{}_ud.svg", base));
    write_to_svg(&make_flip_ud_part(&make_flip_lr_part(part)), &format!("{}_lr_ud.svg", base));
}