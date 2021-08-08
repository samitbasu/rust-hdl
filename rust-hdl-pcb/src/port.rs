use crate::bom::Manufacturer;
use crate::circuit::{CircuitNode, PartDetails};
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, EdgeLocation, PinKind, PinLocation};
use crate::glyph::{Glyph, Point, Rect};
use crate::smd::SizeCode;
use crate::utils::pin_list;

const CHAR_WIDTH: i32 = 55;
const PORT_HALF_HEIGHT: i32 = 55;

pub fn make_port(name: &str, kind: PinKind) -> CircuitNode {
    let pin = EPin {
        kind,
        name: name.to_string(),
        location: PinLocation {
            offset: 0,
            edge: EdgeLocation::East,
        },
    };
    let label = pin.name.len() as i32;
    CircuitNode::Port(PartDetails {
        label: "".to_string(),
        manufacturer: Manufacturer {
            name: "".to_string(),
            part_number: "".to_string(),
        },
        description: "".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        hide_part_outline: false,
        pins: pin_list(vec![pin]),
        outline: vec![Glyph::OutlineRect(Rect {
            p0: Point {
                x: -label * CHAR_WIDTH,
                y: -PORT_HALF_HEIGHT,
            },
            p1: Point {
                x: label * CHAR_WIDTH,
                y: PORT_HALF_HEIGHT,
            },
        })],
        size: SizeCode::Virtual,
    })
}
