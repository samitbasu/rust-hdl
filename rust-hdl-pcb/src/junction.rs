use crate::circuit::{CircuitNode, PartDetails};
use crate::epin::{EPin, PinKind, PinLocation, EdgeLocation};
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::glyph::{Glyph, Rect, Point, Circle};
use crate::utils::pin_list;

const JUNCTION_DISK_RADIUS : f64 = 30.0;

pub fn make_junction() -> CircuitNode {
    let pin = EPin {
        kind: PinKind::Passive,
        name: "".to_string(),
        location: PinLocation {
            offset: 0,
            edge: EdgeLocation::South
        }
    };
    CircuitNode::Junction(PartDetails {
        label: "".to_string(),
        manufacturer: Manufacturer { name: "".to_string(), part_number: "".to_string() },
        description: "".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        hide_part_outline: true,
        pins: pin_list(vec![pin]),
        outline: vec![
            Glyph::Circle(Circle {
                p0: Point::zero(),
                radius: JUNCTION_DISK_RADIUS,
            })
        ],
        suppliers: vec![],
        designator: Designator { kind: DesignatorKind::Resistor, index: None },
        size: SizeCode::Virtual
    })
}