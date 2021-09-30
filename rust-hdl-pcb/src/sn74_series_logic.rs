use rust_hdl_pcb_core::prelude::*;

pub fn make_sn74hct138(part_number: &str) -> CircuitNode {
    assert_eq!(part_number, "SN74HCT138PWR");
    let mut pinset = vec![
        pin!("A", Input, 800, West),
        pin!("B", Input, 600, West),
        pin!("C", Input, 400, West),
        pin!("~G2A", InputInverted, -200, West),
        pin!("~G2B", InputInverted, -400, West),
        pin!("G1", Input, 0, West),
        pin!("Y7", Output, -800, East),
        pin!("GND", PowerReturn, -800, West),
    ];
    for i in 0..=6 {
        pinset.push(pin!(&format!("Y{}", 6 - i), Output, 600 - 200 * i, East));
    }
    pinset.push(pin!("VCC", PowerSink, 900, East));
    CircuitNode::Logic(Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer {
                name: "TI".to_string(),
                part_number: part_number.into(),
            },
            description: "3-to-8 Decoder/Demux".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(pinset),
            outline: vec![
                make_ic_body(-600, -900, 600, 1000),
                make_label(-600, 1000, "U?", TextJustification::BottomLeft),
                make_label(-600, -900, part_number, TextJustification::TopLeft),
            ],
            size: SizeCode::TSSOP(16),
        },
        drive_current_ma: 4.0,
        min_supply_voltage: 4.5,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::TTL,
        output_type: LogicSignalStandard::TTL,
        function: LogicFunction::Decoder,
    })
}

pub fn make_sn74hct541(part_number: &str) -> CircuitNode {
    assert_eq!(part_number, "SN74HCT541PWR");
    let mut pinset = vec![];
    pinset.push(pin!("~OE1", InputInverted, 900, West));
    for i in 1..=8 {
        pinset.push(pin!(&format!("A{}", i), Input, 800 - i * 200, West));
    }
    pinset.push(pin!("GND", PowerReturn, -1100, East));
    for i in 1..=8 {
        pinset.push(pin!(
            &format!("Y{}", 9 - i),
            TriState,
            -1000 + i * 200,
            East
        ));
    }
    pinset.push(pin!("~OE2", InputInverted, -1100, West));
    pinset.push(pin!("VCC", PowerSink, 900, East));
    CircuitNode::Logic(Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer {
                name: "TI".to_string(),
                part_number: part_number.into(),
            },
            description: "Octal Buffer and Line Driver, 3-State outputs".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(pinset),
            outline: vec![
                make_ic_body(-700, -1200, 600, 1000),
                make_label(-700, 1000, "U?", TextJustification::BottomLeft),
                make_label(-700, -1200, part_number, TextJustification::TopLeft),
            ],
            size: SizeCode::TSSOP(20),
        },
        drive_current_ma: 6.0,
        min_supply_voltage: 4.5,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::TTL,
        output_type: LogicSignalStandard::TriState5v0,
        function: LogicFunction::Buffer,
    })
}

pub fn make_sn74hc151(part_number: &str) -> CircuitNode {
    assert_eq!(part_number, "SN74HC151QDRQ1");
    let mut pins = vec![];
    for i in 0..=3 {
        pins.push(pin!(&format!("D{}", 3 - i), Input, 400 + 200 * i, West));
    }
    pins.push(pin!("Y", Output, 700, East));
    pins.push(pin!("W", Output, 400, East));
    pins.push(pin!("~G", InputInverted, 300, South));
    pins.push(pin!("GND", PowerReturn, -400, East));
    pins.push(pin!("C", Input, -400, South));
    pins.push(pin!("B", Input, -200, South));
    pins.push(pin!("A", Input, 0, South));
    for i in 0..=3 {
        pins.push(pin!(&format!("D{}", 7 - i), Input, -400 + 200 * i, West));
    }
    pins.push(pin!("VCC", PowerSink, 1100, East));
    CircuitNode::Logic(Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer {
                name: "TI".to_string(),
                part_number: part_number.into(),
            },
            description: "8-Line to 1-Line Multiplexer".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(pins),
            outline: vec![
                make_ic_body(-500, -700, 400, 1200),
                make_label(-500, 1200, "U?", TextJustification::BottomLeft),
                make_label(-300, 1200, part_number, TextJustification::BottomLeft),
            ],
            size: SizeCode::SOIC(16),
        },
        drive_current_ma: 6.0,
        min_supply_voltage: 2.0,
        max_supply_voltage: 6.0,
        input_type: LogicSignalStandard::TTL,
        output_type: LogicSignalStandard::TTL,
        function: LogicFunction::Multiplexer,
    })
}

pub fn make_sn74_series(part_number: &str) -> CircuitNode {
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
