use crate::circuit::PartDetails;
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;
use crate::isolators::make_iso7741edwrq1;
use crate::glyph::{Glyph, Rect};
use crate::epin::{EPin, EdgeLocation};
use svg::node::element::Text;

pub fn add_pins(mut doc: Document, part: &PartDetails) -> Document {
    if part.outline.len() == 0 {
        return doc
    }
    if let Glyph::OutlineRect(r) = &part.outline[0] {
        for pin in &part.pins {
            match pin.1.location.edge {
                EdgeLocation::North => {}
                EdgeLocation::East => {
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
                EdgeLocation::South => {}
                EdgeLocation::West => {
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

pub fn outline_to_path(g: &Glyph) -> Path {
    match g {
        Glyph::OutlineRect(r) => {
            let data = Data::new()
                .move_to((r.p0.x, -r.p0.y))
                .line_to((r.p0.x, -r.p1.y))
                .line_to((r.p1.x, -r.p1.y))
                .line_to((r.p1.x, -r.p0.y))
                .close();
            Path::new()
                .set("fill", "#FFFDB0")
                .set("stroke", "#AE5E46")
                .set("stroke-width", 5)
                .set("d", data)
        },
        Glyph::Line(l) => {
            let data = Data::new()
                .move_to((l.p0.x, -l.p0.y))
                .line_to((l.p1.x, -l.p1.y));
            Path::new()
                .set("fill", "none")
                .set("stroke", "#0433FF")
                .set("stroke-width", 10)
                .set("d", data)
        }
    }

}


// Color for the body of an IC: FFFDB0
// Color for the schematic line: AE5E46
pub fn make_svg(part: &PartDetails) {
/*    let data = Data::new()
        .move_to((10, 10))
        .line_by((0, 50))
        .line_by((50, 0))
        .line_by((0, -50))
        .close();


 */

    let mut document = Document::new()
        .set("viewBox", (-2000, -2000, 4000, 4000));
    for x in &part.outline {
        document = document.add(outline_to_path(x));
    }
    document = add_pins(document, &part);

    svg::save("test.svg", &document).unwrap();
}

#[test]
fn test_svg_of_part() {
    let u = make_iso7741edwrq1("ISO7741EDWRQ1");
    make_svg(&u);
}