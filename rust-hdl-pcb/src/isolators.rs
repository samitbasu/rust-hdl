use crate::circuit::PartDetails;
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::epin::{EPin, PinKind};
use crate::utils::pin_list;

pub fn make_iso7741edwrq1(part_number: &str) -> PartDetails {
    assert_eq!(part_number, "ISO7741EDWRQ1");
    let pins = vec![
        EPin::new("VCC1", PinKind::PowerSink),
        EPin::new("GND1_1", PinKind::PowerReturn),
        EPin::new("INA", PinKind::Input),
        EPin::new("INB", PinKind::Input),
        EPin::new("INC", PinKind::Input),
        EPin::new("OUTD", PinKind::Output),
        EPin::new("EN1", PinKind::Input),
        EPin::new("GND1_2", PinKind::PowerReturn),
        EPin::new("GND2_2", PinKind::PowerReturn),
        EPin::new("EN2", PinKind::Input),
        EPin::new("IND", PinKind::Input),
        EPin::new("OUTC", PinKind::Output),
        EPin::new("OUTB", PinKind::Output),
        EPin::new("OUTA", PinKind::Output),
        EPin::new("GND2_1", PinKind::PowerReturn),
        EPin::new("VCC2", PinKind::PowerSink),
    ];
    PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer { name: "TI".to_string(), part_number: part_number.into() },
        description: "Quad Channel Digital Isolator - Automotive Grade 0".to_string(),
        comment: "".to_string(),
        hide_pin_designators: false,
        pins: pin_list(pins),
        suppliers: vec![],
        designator: Designator { kind: DesignatorKind::IntegratedCircuit, index: None },
        size: SizeCode::SOIC(16)
    }
}