use crate::bom::Manufacturer;
use crate::capacitors::map_three_digit_cap_to_pf;
use crate::circuit::{CircuitNode, Inductor, PartDetails};
use crate::designator::{Designator, DesignatorKind};
use crate::epin::make_passive_pin_pair;
use crate::glyph::{make_arc, make_ic_body, make_label, TextJustification};
use crate::smd::SizeCode;
use crate::utils::pin_list;

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
    let label = format!(
        "{}uH {}mA {}R",
        value_microhenry, max_current_milliamps, dc_resistance_ohms
    );
    let mut outline = vec![make_ic_body(-200, 0, 200, 50)];
    outline.extend(
        (0..=3)
            .into_iter()
            .map(|x| make_arc(-150 + x * 100, 0, 50.0, -179.9, 179.9))
            .collect::<Vec<_>>(),
    );
    let line1: String = label.split(" ").take(2).collect::<Vec<_>>().join(" ");
    let line2: String = label.split(" ").skip(2).collect::<Vec<_>>().join(" ");
    outline.extend(vec![
        make_label(-200, 70, "L?", TextJustification::BottomLeft),
        make_label(-300, -30, &line1, TextJustification::TopLeft),
        make_label(-300, -130, &line2, TextJustification::TopLeft),
    ]);
    CircuitNode::Inductor(Inductor {
        details: PartDetails {
            label,
            manufacturer: Manufacturer {
                name: "Taiyo Yuden".to_string(),
                part_number: part_number.into(),
            },
            description: "".to_string(),
            comment: "".to_string(),
            hide_pin_designators: true,
            hide_part_outline: true,
            pins: pin_list(make_passive_pin_pair()),
            outline,
            size,
        },
        value_microhenry,
        tolerance,
        dc_resistance_ohms,
        max_current_milliamps,
    })
}
