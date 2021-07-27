use crate::circuit::{PartDetails, Logic, LogicSignalStandard, LogicFunction};
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::epin::{EPin, PinKind};

pub fn make_sn74lvc1g125se7(part_number: &str) -> Logic {
    assert_eq!(part_number, "74LVC1G125SE-7");
    Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer { name: "Diodes".to_string(), part_number: part_number.into() },
            description: "Single Buffer Gate with 3-state output".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(vec![
                EPin {
                    name: "OE".into(),
                    kind: PinKind::InputInverted,
                },
                EPin {
                    name: "A".into(),
                    kind: PinKind::Input,
                },
                EPin {
                    name: "GND".into(),
                    kind: PinKind::PowerReturn,
                },
                EPin {
                    name: "Y".into(),
                    kind: PinKind::TriState,
                },
                EPin {
                    name: "VCC".into(),
                    kind: PinKind::PowerSink,
                }
            ]),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::Resistor, index: None },
            size: SizeCode::SOT353,
        },
        drive_current_ma: 24.0,
        min_supply_voltage: 1.65,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::WideRange,
        output_type: LogicSignalStandard::TriState,
        function: LogicFunction::Buffer
    }
}

pub fn make_sn74lvc1g86dck(part_number: &str) -> Logic {
    assert_eq!(part_number, "SN74LVC1G86DCK");
    Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer { name: "TI".into(), part_number: part_number.into() },
            description: "Single 2-input XOR Gate".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(vec![
                EPin {
                    kind: PinKind::Input,
                    name: "A".to_string(),
                },
                EPin {
                    kind: PinKind::Input,
                    name: "B".to_string(),
                },
                EPin {
                    kind: PinKind::PowerReturn,
                    name: "GND".to_string(),
                },
                EPin {
                    kind: PinKind::Output,
                    name: "Y".to_string(),
                },
                EPin {
                    kind: PinKind::PowerSink,
                    name: "VCC".to_string(),
                }
            ]),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::IntegratedCircuit, index: None },
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