use rust_hdl_pcb_core::prelude::*;

pub fn make_on_semi_ncv33375_regulator(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("NCV33375ST"));
    let voltage = match &part_number[10..=12] {
        "1.8" => 1.8,
        "2.5" => 2.5,
        "3.0" => 3.0,
        "3.3" => 3.3,
        "5.0" => 5.0,
        _ => panic!("Unexpected voltage in part {}", part_number),
    };
    CircuitNode::Regulator(Regulator {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer {
                name: "ON Semiconductor".to_string(),
                part_number: part_number.into(),
            },
            description: "300mA LDO Automotive, 13V input range".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(vec![
                pin!("VIN", PowerSink, 200, West),
                pin!("ON/OFF", Input, -100, West),
                pin!("VOUT", PowerSource, 200, East),
                pin!("GND", PowerReturn, 100, South),
            ]),
            outline: vec![
                make_ic_body(-400, -200, 500, 400),
                make_label(-400, 400, "V?", TextJustification::BottomLeft),
                make_label(-200, 400, part_number, TextJustification::BottomLeft),
            ],
            size: SizeCode::SOT223,
        },
        input_min_voltage: 0.260 + voltage,
        input_max_voltage: 13.0,
        output_nominal_voltage: voltage,
        output_max_current_ma: 300.0,
    })
}

pub fn make_mcp_1799_regulator(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("MCP1799"));
    let voltage = match &part_number[9..=10] {
        "33" => 3.3,
        "50" => 5.0,
        _ => panic!("unwknown part number"),
    };
    CircuitNode::Regulator(Regulator {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer {
                name: "Microchip".to_string(),
                part_number: part_number.into(),
            },
            description: "80mA LDO Automotive, 45V input range".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(vec![
                pin!("VIN", PowerSink, 100, West),
                pin!("GND_1", PowerReturn, -200, West),
                pin!("VOUT", PowerSource, 100, East),
                pin!("GND_2", PowerReturn, -200, East),
            ]),
            outline: vec![
                make_ic_body(-400, -300, 500, 200),
                make_label(-400, 200, "V?", TextJustification::BottomLeft),
                make_label(-400, -300, part_number, TextJustification::TopLeft),
            ],
            size: SizeCode::SOT223,
        },
        input_min_voltage: 0.3 + voltage,
        input_max_voltage: 45.0,
        output_nominal_voltage: voltage,
        output_max_current_ma: 80.0,
    })
}

pub fn make_ti_tps_7b84_regulator(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("TPS7B84"));
    let voltage = match &part_number[7..=8] {
        "33" => 3.3,
        "50" => 5.0,
        _ => panic!("unknown part number"),
    };
    CircuitNode::Regulator(Regulator {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer {
                name: "TI".to_string(),
                part_number: part_number.into(),
            },
            description: "150mA LDO, 40V input range".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(vec![
                pin!("IN", PowerSink, 200, West),
                pin!("EN", Input, -100, West),
                pin!("OUT", PowerSource, 200, East),
                pin!("GND", PowerReturn, 0, South),
            ]),
            outline: vec![
                make_ic_body(-500, -200, 500, 300),
                make_label(-500, 300, "V?", TextJustification::BottomLeft),
                make_label(-300, 300, part_number, TextJustification::BottomLeft),
            ],
            size: SizeCode::SOT223,
        },
        input_min_voltage: 0.3 + voltage,
        input_max_voltage: 42.0,
        output_nominal_voltage: voltage,
        output_max_current_ma: 150.0,
    })
}

pub fn make_zldo1117g_regulator(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("ZLDO1117G"));
    assert!(part_number.ends_with("TA"));
    let voltage = match &part_number[9..=10] {
        "12" => 1.2,
        "15" => 1.5,
        "18" => 1.8,
        "25" => 2.5,
        "33" => 3.3,
        "50" => 5.0,
        _ => panic!("Unrecognized part number {}", part_number),
    };
    CircuitNode::Regulator(Regulator {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer {
                name: "Diodes".to_string(),
                part_number: part_number.into(),
            },
            description: "1A LDO, 18V input range".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(vec![
                pin!("GND", PowerReturn, 0, South),
                pin!("Vout_1", PowerSource, 300, East),
                pin!("Vin", PowerSink, 300, West),
                pin!("Vout_2", PowerSource, 100, East),
            ]),
            outline: vec![
                make_ic_body(-400, -300, 400, 400),
                make_label(-400, 400, "V?", TextJustification::BottomLeft),
                make_label(-200, 400, part_number, TextJustification::BottomLeft),
            ],
            size: SizeCode::SOT223,
        },
        input_min_voltage: 2.7 + voltage,
        input_max_voltage: 18.0,
        output_nominal_voltage: voltage,
        output_max_current_ma: 1000.0,
    })
}
