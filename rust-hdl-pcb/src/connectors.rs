use crate::circuit::PartDetails;
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::epin::{EPin, PinKind};

pub fn make_sullins_sbh11_header(part_number: &str) -> PartDetails {
    assert_eq!(part_number, "SBH11-PBPC-D13-RA-BK"); // TODO - generalize to other pin counts
    PartDetails {
        label: "26P, 2.54mm, RA".into(),
        manufacturer: Manufacturer { name: "Sullins Connector Solutions".to_string(), part_number: part_number.into()},
        description: "Connector/Header 26 Pos, 2.54mm, Right Angle".to_string(),
        comment: "".to_string(),
        hide_pin_designators: true,
        pins: pin_list(
            (0..26).into_iter()
                .map(|x| {
                    EPin::new(&format!("{}", x+1), PinKind::Passive)
                })
                .collect()
        ),
        suppliers: vec![],
        designator: Designator { kind: DesignatorKind::Connector, index: None },
        size: SizeCode::Custom("PTH, Right Angle".into())
    }

}