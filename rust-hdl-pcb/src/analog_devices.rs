use crate::bom::Manufacturer;
use crate::circuit::PartDetails;
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, PinKind};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::pin;

pub fn make_lt3092_current_source(part_number: &str) -> PartDetails {
    assert!(part_number.starts_with("LT3092"));
    PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer {
            name: "Analog Devices".to_string(),
            part_number: part_number.into(),
        },
        description: "Programmable Current Source/Limiter".to_string(),
        comment: "".to_string(),
        hide_pin_designators: false,
        pins: pin_list(vec![
            pin!("SET", Input, -100, West),
            pin!("OUT_1", PowerSource, -100, East),
            pin!("IN", PowerSink, 200, West),
            pin!("OUT_2", PowerSource, 200, East),
        ]),
        suppliers: vec![],
        designator: Designator {
            kind: DesignatorKind::IntegratedCircuit,
            index: None,
        },
        size: SizeCode::SOT223,
    }
}
