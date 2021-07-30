use crate::bom::Manufacturer;
use crate::circuit::{Diode, PartDetails, CircuitNode};
use crate::designator::{Designator, DesignatorKind};
use crate::diode::DiodeKind;
use crate::epin::{EPin, EdgeLocation};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::glyph::{make_pin, make_line};

pub fn make_wurth_led(part_number: &str) -> CircuitNode {
    // Only one supported type for now...
    assert_eq!(part_number, "150060GS75000");
    CircuitNode::Diode(Diode {
        details: PartDetails {
            label: "Green LED".to_string(),
            manufacturer: Manufacturer {
                name: "Wurth".to_string(),
                part_number: part_number.to_owned(),
            },
            description: "Green 520nm LED Indication - Discrete 3.2V".to_string(),
            comment: "".to_string(),
            hide_pin_designators: true,
            pins: pin_list(vec![EPin::passive_neg(), EPin::passive_pos()]),
            outline: vec![
                make_pin(200, 0, EdgeLocation::East, 100),
                make_pin(-100, 0, EdgeLocation::West, 100),
                make_line(-100, 0, -50, 0),
                make_line(-50, 70, -50, -70),
                make_line(-50, -70, 70, 0),
                make_line(70, 0, -50, 70),
                make_line(60, 0, 200, 0),
                make_line(70, 70, 70, -70),
                make_line(30, 90, 90, 150),
                make_line(50, 150, 90, 150),
                make_line(90, 150, 90, 110),
                make_line(-20, 140, 40, 200),
                make_line(0, 200, 40, 200),
                make_line(40, 200, 40, 160),
            ],
            suppliers: vec![],
            designator: Designator {
                kind: DesignatorKind::Diode,
                index: None,
            },
            size: SizeCode::I0603,
        },
        forward_drop_volts: 3.2,
        kind: DiodeKind::LED("Green".into()),
    })
}
