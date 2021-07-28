use crate::bom::Manufacturer;
use crate::circuit::PartDetails;
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, PinKind};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::pin;

pub fn make_iso7741edwrq1(part_number: &str) -> PartDetails {
    assert_eq!(part_number, "ISO7741EDWRQ1");
    let pins = vec![
        pin!("VCC1", PowerSink, 800, West),
        pin!("GND1_1", PowerReturn, -900, West),
        pin!("INA", Input, 300, West),
        pin!("INB", Input, 100, West),
        pin!("INC", Input, -100, West),
        pin!("OUTD", Output, -300, West),
        pin!("EN1", Input, 600, West),
        pin!("GND1_2", PowerReturn, -700, West),
        pin!("GND2_2", PowerReturn, -700, East),
        pin!("EN2", Input, 600, East),
        pin!("IND", Input, -300, East),
        pin!("OUTC", Output, -100, East),
        pin!("OUTB", Output, 100, East),
        pin!("OUTA", Output, 300, East),
        pin!("GND2_1", PowerReturn, -900, East),
        pin!("VCC2", PowerSink, 800, East),
    ];
    PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer {
            name: "TI".to_string(),
            part_number: part_number.into(),
        },
        description: "Quad Channel Digital Isolator - Automotive Grade 0".to_string(),
        comment: "".to_string(),
        hide_pin_designators: false,
        pins: pin_list(pins),
        suppliers: vec![],
        designator: Designator {
            kind: DesignatorKind::IntegratedCircuit,
            index: None,
        },
        size: SizeCode::SOIC(16),
    }
}
