use rust_hdl_pcb_core::prelude::*;
use std::collections::BTreeMap;

fn make_passive_pins(count: u32, start_y: i32, delta_y: i32) -> BTreeMap<u64, EPin> {
    pin_list(
        (0..count)
            .into_iter()
            .map(|x| {
                EPin::new(
                    &format!("{}", x + 1),
                    PinKind::Passive,
                    PinLocation {
                        offset: start_y - delta_y * (x as i32),
                        edge: EdgeLocation::East,
                    },
                )
            })
            .collect(),
    )
}

pub fn make_molex_55935_connector(part_number: &str) -> CircuitNode {
    assert_eq!(part_number, "0559350810"); // TODO - generalize to other pin counts
    CircuitNode::Connector(PartDetails {
        label: "8pos, 2mm MicroClasp".to_string(),
        manufacturer: Manufacturer {
            name: "Molex".to_string(),
            part_number: part_number.into(),
        },
        description: "Connector Shrouded Header 8 pos, 2mm RA Thru-Hole MicroClasp".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        hide_part_outline: false,
        pins: make_passive_pins(8, 300, 100),
        outline: vec![
            make_ic_body(-200, -500, 0, 400),
            make_label(-200, 400, "J?", TextJustification::BottomLeft),
            make_label(-200, -500, part_number, TextJustification::TopLeft),
        ],
        size: SizeCode::Custom("PTH, Right Angle".into()),
    })
}

pub fn make_sullins_sbh11_header(part_number: &str) -> CircuitNode {
    assert_eq!(part_number, "SBH11-PBPC-D13-RA-BK"); // TODO - generalize to other pin counts
    CircuitNode::Connector(PartDetails {
        label: "26P, 2.54mm, RA".into(),
        manufacturer: Manufacturer {
            name: "Sullins Connector Solutions".to_string(),
            part_number: part_number.into(),
        },
        description: "Connector/Header 26 Pos, 2.54mm, Right Angle".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        hide_part_outline: false,
        pins: make_passive_pins(26, 1200, 100),
        outline: vec![make_ic_body(-200, -1400, 0, 1300)],
        size: SizeCode::Custom("PTH, Right Angle".into()),
    })
}

pub fn make_amphenol_10056845_header(part_number: &str) -> CircuitNode {
    assert_eq!(part_number, "10056845-108LF"); // TODO - generalize to other pin counts
    CircuitNode::Connector(PartDetails {
        label: "8P, 2.54mm, RA".into(),
        manufacturer: Manufacturer {
            name: "Amphenol".to_string(),
            part_number: part_number.into(),
        },
        description: "Connector/Header 8 Pos, 2.54mm, Right Angle".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        hide_part_outline: false,
        pins: pin_list(vec![
            pin!("1", Passive, 300, West),
            pin!("2", Passive, 300, East),
            pin!("3", Passive, 100, West),
            pin!("4", Passive, 100, East),
            pin!("5", Passive, -100, West),
            pin!("6", Passive, -100, East),
            pin!("7", Passive, -300, West),
            pin!("8", Passive, -300, East),
        ]),
        outline: vec![
            make_ic_body(-200, -400, 300, 400),
            make_label(-200, 400, "J?", TextJustification::BottomLeft),
            make_label(-200, -400, part_number, TextJustification::TopLeft),
        ],
        size: SizeCode::Custom("PTH, Right Angle".into()),
    })
}
