use rust_hdl_pcb_core::prelude::*;

pub fn make_sn74lvc1g125se7(part_number: &str) -> CircuitNode {
    assert_eq!(part_number, "74LVC1G125SE-7");
    CircuitNode::Logic(Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer {
                name: "Diodes".to_string(),
                part_number: part_number.into(),
            },
            description: "Single Buffer Gate with 3-state output".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(vec![
                pin!("~OE", InputInverted, 200, West),
                pin!("A", Input, 0, West),
                pin!("GND", PowerReturn, -200, East),
                pin!("Y", TriState, 0, East),
                pin!("VCC", PowerSink, 200, East),
            ]),
            outline: vec![
                make_ic_body(-400, -400, 400, 400),
                make_label(-400, 400, "U?", TextJustification::BottomLeft),
                make_label(-400, -400, part_number, TextJustification::TopLeft),
            ],
            size: SizeCode::SOT353,
        },
        drive_current_ma: 24.0,
        min_supply_voltage: 1.65,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::WideRange,
        output_type: LogicSignalStandard::TriState,
        function: LogicFunction::Buffer,
    })
}

pub fn make_sn74lvc1g86dck(part_number: &str) -> CircuitNode {
    assert_eq!(part_number, "SN74LVC1G86DCK");
    CircuitNode::Logic(Logic {
        details: PartDetails {
            label: part_number.into(),
            manufacturer: Manufacturer {
                name: "TI".into(),
                part_number: part_number.into(),
            },
            description: "Single 2-input XOR Gate".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            hide_part_outline: false,
            pins: pin_list(vec![
                pin!("A", Input, 100, West),
                pin!("B", Input, -100, West),
                pin!("GND", PowerReturn, -300, East),
                pin!("Y", Output, 0, East),
                pin!("VCC", PowerSink, 300, East),
            ]),
            outline: vec![
                make_ic_body(-500, -500, 600, 500),
                make_label(-500, 500, "U?", TextJustification::BottomLeft),
                make_label(-500, -500, part_number, TextJustification::TopLeft),
                make_arc(-560, 0, 400.0, 330.0, 60.0),
                make_arc(-490, 0, 400.0, 330.0, 60.0),
                make_line(-140, 200, 0, 200),
                make_line(-140, -200, 0, -200),
                make_arc(0, 180, 380.0, 270.0, 60.0),
                make_arc(0, -180, 380.0, 30.0, 60.0),
                make_line(-270, 100, -180, 100),
                make_line(-270, -100, -180, -100),
                make_line(330, 0, 400, 0),
            ],
            size: SizeCode::SC70,
        },
        drive_current_ma: 32.0,
        min_supply_voltage: 1.65,
        max_supply_voltage: 5.5,
        input_type: LogicSignalStandard::WideRange,
        output_type: LogicSignalStandard::WideRange,
        function: LogicFunction::XOR,
    })
}

pub fn make_lvc_one_gate(part_number: &str) -> CircuitNode {
    if part_number.starts_with("74LVC1G125") {
        make_sn74lvc1g125se7(part_number)
    } else if part_number.starts_with("SN74LVC1G86") {
        make_sn74lvc1g86dck(part_number)
    } else {
        unimplemented!("No part for that type yet {}", part_number);
    }
}
