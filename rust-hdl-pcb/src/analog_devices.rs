use crate::circuit::PartDetails;
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::epin::{EPin, PinKind, SignalKind};

pub fn make_lt3092_current_source(part_number: &str) -> PartDetails {
    assert!(part_number.starts_with("LT3092"));
    PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer { name: "Analog Devices".to_string(), part_number: part_number.into() },
        description: "Programmable Current Source/Limiter".to_string(),
        comment: "".to_string(),
        pins: pin_list(vec![
            EPin {
                kind: PinKind::Input(SignalKind::Any),
                name: "SET".to_string(),
                designator_visible: true
            },
            EPin {
                kind: PinKind::PowerSource,
                name: "OUT_1".to_string(),
                designator_visible: true
            },
            EPin {
                kind: PinKind::PowerSink,
                name: "IN".to_string(),
                designator_visible: true
            },
            EPin {
                kind: PinKind::PowerSource,
                name: "OUT_2".to_string(),
                designator_visible: true,
            }
        ]),
        suppliers: vec![],
        designator: Designator { kind: DesignatorKind::IntegratedCircuit, index: None },
        size: SizeCode::SOT223,
    }

}