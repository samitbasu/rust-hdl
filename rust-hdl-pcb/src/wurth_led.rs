use crate::bom::Manufacturer;
use crate::circuit::{Diode, PartDetails};
use crate::designator::{Designator, DesignatorKind};
use crate::diode::DiodeKind;
use crate::epin::EPin;
use crate::smd::SizeCode;
use crate::utils::pin_list;

pub fn make_wurth_led(part_number: &str) -> Diode {
    // Only one supported type for now...
    assert_eq!(part_number, "150060GS75000");
    Diode {
        details: PartDetails {
            label: "Green LED".to_string(),
            manufacturer: Manufacturer {
                name: "".to_string(),
                part_number: "".to_string(),
            },
            description: "Green 520nm LED Indication - Discrete 3.2V".to_string(),
            comment: "".to_string(),
            hide_pin_designators: true,
            pins: pin_list(vec![EPin::passive_neg(), EPin::passive_pos()]),
            suppliers: vec![],
            designator: Designator {
                kind: DesignatorKind::Diode,
                index: None,
            },
            size: SizeCode::I0603,
        },
        forward_drop_volts: 3.2,
        kind: DiodeKind::LED("Green".into()),
    }
}
