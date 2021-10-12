use rust_hdl_pcb_core::prelude::*;

pub fn make_lt3092_current_source(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("LT3092"));
    CircuitNode::IntegratedCircuit(PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer {
            name: "Analog Devices".to_string(),
            part_number: part_number.into(),
        },
        description: "Programmable Current Source/Limiter".to_string(),
        comment: "".to_string(),
        hide_pin_designators: false,
        hide_part_outline: false,
        pins: pin_list(vec![
            pin!("SET", Input, -100, West),
            pin!("OUT_1", PowerSource, -100, East),
            pin!("IN", PowerSink, 200, West),
            pin!("OUT_2", PowerSource, 200, East),
        ]),
        outline: vec![
            make_ic_body(-400, -200, 400, 300),
            make_label(-400, 300, "U?", TextJustification::BottomLeft),
            make_label(-400, -200, part_number, TextJustification::TopLeft),
        ],
        size: SizeCode::SOT223,
    })
}
