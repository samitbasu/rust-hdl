use crate::circuit::{Logic, PartDetails, LogicSignalStandard, LogicFunction};
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::epin::{EPin, PinKind};
use crate::utils::pin_list;

pub fn make_sn74hct138(part_number: &str) -> Logic {
    assert_eq!(part_number, "SN74HCT138PWR");
    let mut pinset = vec![
        EPin::new("A", PinKind::Input),
        EPin::new("B", PinKind::Input),
        EPin::new("C", PinKind::Input),
        EPin::new("G2A", PinKind::InputInverted),
        EPin::new("G2B", PinKind::InputInverted),
        EPin::new("G1", PinKind::Input),
        EPin::new("Y7", PinKind::Output),
        EPin::new("GND", PinKind::PowerReturn),
    ];
    for i in 0..=6 {
        pinset.push(EPin::new(&format!("Y{}", 6-i), PinKind::Output));
    }
    pinset.push(EPin::new("VCC", PinKind::PowerSink));
    Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer { name: "TI".to_string(), part_number: part_number.into() },
            description: "3-to-8 Decoder/Demux".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(pinset),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::IntegratedCircuit, index: None },
            size: SizeCode::TSSOP(16)
        },
        drive_current_ma: 4.0,
        min_supply_voltage: 4.5,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::TTL,
        output_type: LogicSignalStandard::TTL,
        function: LogicFunction::Decoder,
    }
}

pub fn make_sn74hct541(part_number: &str) -> Logic {
    assert_eq!(part_number, "SN74HCT541PWR");
    let mut pinset = vec![];
    pinset.push(EPin::new("OE1", PinKind::InputInverted));
    for i in 1..=8 {
        pinset.push(EPin::new(&format!("A{}", i), PinKind::Input));
    }
    pinset.push(EPin::new("GND", PinKind::PowerReturn));
    for i in 1..=8 {
        pinset.push(EPin::new(&format!("Y{}", 9-i), PinKind::TriState));
    }
    pinset.push(EPin::new("OE2", PinKind::InputInverted));
    pinset.push(EPin::new("VCC", PinKind::PowerSink));
    Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer { name: "TI".to_string(), part_number: part_number.into() },
            description: "Octal Buffer and Line Driver, 3-State outputs".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(pinset),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::IntegratedCircuit, index: None },
            size: SizeCode::TSSOP(20)
        },
        drive_current_ma: 6.0,
        min_supply_voltage: 4.5,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::TTL,
        output_type: LogicSignalStandard::TriState5v0,
        function: LogicFunction::Buffer
    }
}

pub fn make_sn74hc151(part_number: &str) -> Logic {
    assert_eq!(part_number, "SN74HC151QDRQ1");
    let mut pins = vec![];
    for i in 0..=3 {
        pins.push(EPin::new(&format!("D{}", 3-i), PinKind::Input));
    }
    pins.push(EPin::new("Y", PinKind::Output));
    pins.push(EPin::new("W", PinKind::Output));
    pins.push(EPin::new("G", PinKind::InputInverted));
    pins.push(EPin::new("GND", PinKind::PowerReturn));
    pins.push(EPin::new("C", PinKind::Input));
    pins.push(EPin::new("B", PinKind::Input));
    pins.push(EPin::new("A", PinKind::Input));
    for i in 0..=3 {
        pins.push(EPin::new(&format!("D{}", 7-i), PinKind::Input));
    }    
    pins.push(EPin::new("VCC", PinKind::PowerSink));
    Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer { name: "TI".to_string(), part_number: part_number.into() },
            description: "8-Line to 1-Line Multiplexer".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(pins),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::IntegratedCircuit, index: None },
            size: SizeCode::SOIC(16)
        },
        drive_current_ma: 6.0,
        min_supply_voltage: 2.0,
        max_supply_voltage: 6.0,
        input_type: LogicSignalStandard::TTL,
        output_type: LogicSignalStandard::TTL,
        function: LogicFunction::Multiplexer
    }
}



pub fn make_sn74_series(part_number: &str) -> Logic {
    if part_number.starts_with("SN74HCT541") {
        make_sn74hct541(part_number)
    } else if part_number.starts_with("SN74HCT138") {
        make_sn74hct138(part_number)
    } else if part_number.starts_with("SN74HC151") {
        make_sn74hc151(part_number)
    } else {
        unimplemented!()
    }
}