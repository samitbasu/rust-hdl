use crate::bom::Manufacturer;
use crate::circuit::{Logic, LogicFunction, LogicSignalStandard, PartDetails};
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, PinKind};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::pin;
use crate::glyph::make_ic_body;

pub fn make_sn74lvc1g125se7(part_number: &str) -> Logic {
    assert_eq!(part_number, "74LVC1G125SE-7");
    Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer {
                name: "Diodes".to_string(),
                part_number: part_number.into(),
            },
            description: "Single Buffer Gate with 3-state output".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(vec![
                pin!("~OE", InputInverted, 200, West),
                pin!("A", Input, 0, West),
                pin!("GND", PowerReturn, -200, East),
                pin!("Y", TriState, 0, East),
                pin!("VCC", PowerSink, 200, East),
            ]),
            outline: vec![make_ic_body(-400, -400, 400, 400)],
            suppliers: vec![],
            designator: Designator {
                kind: DesignatorKind::Resistor,
                index: None,
            },
            size: SizeCode::SOT353,
        },
        drive_current_ma: 24.0,
        min_supply_voltage: 1.65,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::WideRange,
        output_type: LogicSignalStandard::TriState,
        function: LogicFunction::Buffer,
    }
}

pub fn make_sn74lvc1g86dck(part_number: &str) -> Logic {
    assert_eq!(part_number, "SN74LVC1G86DCK");
    Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer {
                name: "TI".into(),
                part_number: part_number.into(),
            },
            description: "Single 2-input XOR Gate".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(vec![
                pin!("A", Input, 100, West),
                pin!("B", Input, -100, West),
                pin!("GND", PowerReturn, -300, East),
                pin!("Y", Output, 0, East),
                pin!("VCC", PowerSink, 300, East)
            ]),
            outline: vec![make_ic_body(-500, -500, 600, 500)],
            suppliers: vec![],
            designator: Designator {
                kind: DesignatorKind::IntegratedCircuit,
                index: None,
            },
            size: SizeCode::SC70,
        },
        drive_current_ma: 32.0,
        min_supply_voltage: 1.65,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::WideRange,
        output_type: LogicSignalStandard::WideRange,
        function: LogicFunction::XOR,
    }
}

pub fn make_lvc_one_gate(part_number: &str) -> Logic {
    if part_number.starts_with("74LVC1G125") {
        make_sn74lvc1g125se7(part_number)
    } else if part_number.starts_with("SN74LVC1G86") {
        make_sn74lvc1g86dck(part_number)
    } else {
        unimplemented!("No part for that type yet {}", part_number);
    }
}
