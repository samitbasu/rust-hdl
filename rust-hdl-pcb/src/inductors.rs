use crate::bom::Manufacturer;
use crate::capacitors::map_three_digit_cap_to_pf;
use crate::circuit::{Inductor, PartDetails, CircuitNode};
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, make_passive_pin_pair, EdgeLocation};
use crate::smd::SizeCode;
use crate::utils::pin_list;
use crate::glyph::{make_pin, make_arc};

// https://www.yuden.co.jp/productdata/catalog/wound07_e.pdf
pub fn make_ty_brl_series(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("BRL"));
    let size = match &part_number[3..=6] {
        "1608" => SizeCode::I0603,
        "2012" => SizeCode::I0805,
        "3225" => SizeCode::I1210,
        _ => panic!("Unsupported part type"),
    };
    let tolerance = if part_number.ends_with("K") {
        10.0
    } else if part_number.ends_with("M") {
        20.0
    } else {
        panic!("Unsupported part type")
    };
    let value_microhenry = map_three_digit_cap_to_pf(&part_number[8..=10]);
    assert_eq!(part_number, "BRL3225T101K"); // Add others in the future...
    let dc_resistance_ohms = 2.5;
    let max_current_milliamps = 250.0;
    let mut outline = vec![
        make_pin(-200, 0, EdgeLocation::West, 100),
        make_pin(200, 0, EdgeLocation::East, 100)
    ];
    outline.extend(
        (0..=3).into_iter()
            .map(|x| make_arc(-150 + x*100, 0, 50.0, 179.9, -179.9))
            .collect::<Vec<_>>());
    CircuitNode::Inductor(Inductor {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer {
                name: "Taiyo Yuden".to_string(),
                part_number: part_number.into(),
            },
            description: "".to_string(),
            comment: "".to_string(),
            hide_pin_designators: true,
            pins: pin_list(make_passive_pin_pair()),
            outline,
            suppliers: vec![],
            designator: Designator {
                kind: DesignatorKind::Inductor,
                index: None,
            },
            size,
        },
        value_microhenry,
        tolerance,
        dc_resistance_ohms,
        max_current_milliamps,
    })
}
