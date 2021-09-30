use rust_hdl_pcb_core::prelude::*;

pub fn make_ads868x(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("ADS868"));
    assert!(part_number.ends_with("IPW"));
    let pins = vec![
        pin!("DGND", PowerReturn, 300, South),
        pin!("AVDD", PowerSink, -200, North),
        pin!("AGND", PowerReturn, -200, South),
        pin!("REFIO", Passive, 0, West),
        pin!("REFGND", PowerReturn, -800, West),
        pin!("REFCAP", Passive, -300, West),
        pin!("AIN_P", Passive, 800, West),
        pin!("AIN_GND", Passive, 400, West),
        pin!("~RST", InputInverted, -900, East),
        pin!("SDI", Input, -700, East),
        pin!("CONVST/~CS", InputInverted, -500, East),
        pin!("SCLK", Input, -300, East),
        pin!("SDO-0", Output, -100, East),
        pin!("ALARM/SDO-1/GPO", Output, 400, East),
        pin!("RVS", Output, 700, East),
        pin!("DVDD", PowerSink, 300, North),
    ];
    CircuitNode::IntegratedCircuit(PartDetails {
        label: part_number.into(),
        manufacturer: Manufacturer {
            name: "TI".to_string(),
            part_number: part_number.into(),
        },
        description: "16-bit high-speed single supply SAR ADC".to_string(),
        comment: "".to_string(),
        hide_pin_designators: false,
        hide_part_outline: false,
        pins: pin_list(pins),
        outline: vec![
            make_ic_body(-800, -1400, 900, 1200),
            make_label(-800, 1200, "U?", TextJustification::BottomLeft),
            make_label(-800, -1400, part_number, TextJustification::TopLeft),
        ],
        size: SizeCode::TSSOP(16),
    })
}
