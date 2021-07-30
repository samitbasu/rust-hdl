use crate::circuit::{PartDetails, CircuitNode};
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;
use crate::isolators::make_iso7741edwrq1;
use crate::glyph::{Glyph, Rect};
use crate::epin::{EPin, EdgeLocation};
use svg::node::element::Text;
use crate::adc::make_ads868x;

pub fn add_pins(mut doc: Document, part: &PartDetails) -> Document {
    if part.outline.len() == 0 {
        return doc
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
                            .set("alignment-baseline", "bottom")
                            .set("transform", format!("rotate(-90, {}, {}) translate({} {})", pn_x, pn_y, pn_x+50, pn_y - 15))
                            .set("font-size", 85);
                        doc = doc.add(txt)
                    }
                    let tx = pin.1.location.offset;
                    let ty = -r.p1.y + 85;
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("text-anchor", "end")
                        .set("alignment-baseline", "middle")
                        .set("transform", format!("rotate(-90, {}, {}) translate({} {})", tx, ty, tx, ty))
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
                            .set("text-anchor", "end")
                            .set("font-size", 85);
                        doc = doc.add(txt);
                    }
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("x", r.p0.x + 85)
                        .set("y", -pin.1.location.offset)
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
                            .set("text-anchor", "end")
                            .set("alignment-baseline", "bottom")
                            .set("transform", format!("rotate(-90, {}, {}) translate({} {})", pn_x, pn_y, pn_x-85, pn_y - 15))
                            .set("font-size", 85);
                        doc = doc.add(txt)
                    }
                    let tx = pin.1.location.offset;
                    let ty = -r.p0.y - 85;
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("text-anchor", "begin")
                        .set("alignment-baseline", "middle")
                        .set("transform", format!("rotate(-90, {}, {}) translate({} {})", tx, ty, tx, ty))
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
                            .set("text-anchor", "begin")
                            .set("font-size", 85);
                        doc = doc.add(txt);
                    }
                    let txt = Text::new()
                        .add(svg::node::Text::new(&pin.1.name))
                        .set("x", r.p1.x - 85)
                        .set("y", -pin.1.location.offset)
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

pub fn add_outline_to_path(doc: Document, g: &Glyph) -> Document {
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
                .set("d", data)
            )
        },
        Glyph::Line(l) => {
            let data = Data::new()
                .move_to((l.p0.x, -l.p0.y))
                .line_to((l.p1.x, -l.p1.y));
            doc.add(
            Path::new()
                .set("fill", "none")
                .set("stroke", "#0433FF")
                .set("stroke-width", 10)
                .set("d", data)
            )
        }
        Glyph::Text(t) => {
            let txt = Text::new()
                .add(svg::node::Text::new(&t.text))
                .set("x", t.p0.x)
                .set("y", -t.p0.y - 15)
                .set("text-anchor", "begin")
                .set("alignment-baseline", "top")
                .set("font-size", 85);
            doc.add(txt)
        }
    }
}


// Color for the body of an IC: FFFDB0
// Color for the schematic line: AE5E46
pub fn make_svg(part: &PartDetails) {
    let mut document = Document::new()
        .set("viewBox", (-2000, -2000, 4000, 4000));
    for x in &part.outline {
        document = add_outline_to_path(document,x);
    }
    document = add_pins(document, &part);

    svg::save(format!("{}.svg", part.manufacturer.part_number.replace("/", "_")), &document).unwrap();
}

#[test]
fn test_svg_of_part() {
    let u = match make_ads868x("ADS8689IPW") {
        CircuitNode::IntegratedCircuit(u) => u,
        _ => panic!("Unexpected node")
    };
    make_svg(&u);
}