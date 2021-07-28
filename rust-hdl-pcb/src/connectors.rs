use crate::circuit::PartDetails;
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::epin::{EPin, PinKind};
use std::collections::BTreeMap;

fn make_passive_pins(count: u32) -> BTreeMap<u64, EPin> {
    pin_list(
        (0..count).into_iter()
            .map(|x| {
                EPin::new(&format!("{}", x+1), PinKind::Passive)
            })
            .collect()
    )
}

pub fn make_molex_55935_connector(part_number: &str) -> PartDetails {
    assert_eq!(part_number, "0559350810"); // TODO - generalize to other pin counts
    PartDetails {
        label: "8pos, 2mm MicroClasp".to_string(),
        manufacturer: Manufacturer { name: "Molex".to_string(), part_number: part_number.into() },
        description: "Connector Shrouded Header 8 pos, 2mm RA Thru-Hole MicroClasp".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        pins: make_passive_pins(8),
        suppliers: vec![],
        designator: Designator { kind: DesignatorKind::Connector, index: None },
        size: SizeCode::Custom("PTH, Right Angle".into())
    }
    
}

pub fn make_sullins_sbh11_header(part_number: &str) -> PartDetails {
    assert_eq!(part_number, "SBH11-PBPC-D13-RA-BK"); // TODO - generalize to other pin counts
    PartDetails {
        label: "26P, 2.54mm, RA".into(),
        manufacturer: Manufacturer { name: "Sullins Connector Solutions".to_string(), part_number: part_number.into()},
        description: "Connector/Header 26 Pos, 2.54mm, Right Angle".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        pins: make_passive_pins(26),
        suppliers: vec![],
        designator: Designator { kind: DesignatorKind::Connector, index: None },
        size: SizeCode::Custom("PTH, Right Angle".into())
    }
}

pub fn make_amphenol_10056845_header(part_number: &str) -> PartDetails {
    assert_eq!(part_number, "10056845-108LF"); // TODO - generalize to other pin counts
    PartDetails {
        label: "8P, 2.54mm, RA".into(),
        manufacturer: Manufacturer {name: "Amphenol".to_string(), part_number: part_number.into()},
        description: "Connector/Header 8 Pos, 2.54mm, Right Angle".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        pins: make_passive_pins(8),
        suppliers: vec![],
        designator: Designator {kind: DesignatorKind::Connector, index: None},
        size: SizeCode::Custom("PTH, Right Angle".into())
    }
}

