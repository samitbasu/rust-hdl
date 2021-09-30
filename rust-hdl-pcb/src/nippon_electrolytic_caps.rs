use rust_hdl_pcb_core::prelude::*;

// Source: https://www.chemi-con.co.jp/products/relatedfiles/capacitor/catalog/HXDRA-e.PDF
fn match_hxd_esr(voltage: f64, cap_pf: f64, size_code: &str) -> i32 {
    let voltage = voltage.floor() as i32;
    let cap = (cap_pf / 1.0e6).floor() as i32;
    match (voltage, cap, size_code) {
        (16, 47, "E61") => 80,
        (16, 82, "F61") => 45,
        (16, 100, "F61") => 45,
        (16, 150, "F80") => 27,
        (16, 180, "F80") => 27,
        (16, 270, "HA0") => 22,
        (16, 330, "HA0") => 22,
        (16, 470, "JA0") => 18,
        (16, 560, "JA0") => 18,
        (25, 33, "E61") => 80,
        (25, 47, "F61") => 50,
        (25, 56, "F61") => 50,
        (25, 68, "F80") => 30,
        (25, 100, "F80") => 30,
        (25, 150, "HA0") => 27,
        (25, 220, "HA0") => 27,
        (25, 270, "JA0") => 20,
        (25, 330, "JA0") => 20,
        (25, 390, "JA0") => 20,
        (35, 22, "E61") => 100,
        (35, 27, "F61") => 60,
        (35, 47, "F61") => 60,
        (35, 47, "F80") => 35,
        (35, 68, "F80") => 35,
        (35, 100, "HA0") => 27,
        (35, 150, "HA0") => 27,
        (35, 150, "JA0") => 20,
        (35, 270, "JA0") => 20,
        (50, 10, "F61") => 80,
        (50, 15, "F80") => 40,
        (50, 22, "F61") => 80,
        (50, 33, "F80") => 40,
        (50, 33, "HA0") => 30,
        (50, 47, "HA0") => 30,
        (50, 56, "JA0") => 25,
        (50, 68, "HA0") => 30,
        (50, 82, "HA0") => 30,
        (50, 100, "JA0") => 25,
        (50, 120, "JA0") => 25,
        (63, 6, "F61") => 120,
        (63, 10, "F61") => 120,
        (63, 10, "F80") => 80,
        (63, 22, "F80") => 80,
        (63, 22, "HA0") => 40,
        (63, 33, "HA0") => 40,
        (63, 33, "JA0") => 30,
        (63, 47, "HA0") => 40,
        (63, 56, "JA0") => 30,
        (63, 82, "JA0") => 30,
        (63, 100, "JA0") => 30,
        (80, 56, "JA0") => 33,
        _ => unimplemented!(
            "Unrecognized voltage, cap size ({} {} {}) triplet",
            voltage,
            cap,
            size_code
        ),
    }
}

pub fn make_nippon_hxd_capacitor(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("HHXD"));
    let voltage = (&part_number[4..=6]).parse::<f64>().unwrap() / 10.0;
    assert_eq!(&part_number[7..8], "A");
    let value_pf = map_three_digit_cap_to_uf(&part_number[10..=12]) * 1.0e6;
    let value = map_pf_to_label(value_pf);
    assert_eq!(&part_number[13..14], "M");
    let tolerance = CapacitorTolerance::TwentyPercent;
    let size = SizeCode::Custom(part_number[14..=16].to_owned());
    let esr = match_hxd_esr(voltage, value_pf, &part_number[14..=16]);
    let label = format!("{} {} {}V {}mR", value, tolerance, voltage, esr);
    let description = format!(
        "United Chemi-Con HXD Seris Alum Poly SMD {} {}",
        size, label
    );
    CircuitNode::Capacitor(Capacitor {
        details: PartDetails {
            label: label.clone(),
            manufacturer: Manufacturer {
                name: "United Chemi-Con".to_string(),
                part_number: part_number.to_owned(),
            },
            description,
            comment: "".to_string(),
            hide_pin_designators: true,
            hide_part_outline: true,
            pins: pin_list(vec![EPin::passive_pos(), EPin::passive_neg()]),
            outline: make_polarized_capacitor_outline(&label),
            size,
        },
        value_pf,
        kind: CapacitorKind::AluminumPolyLowESR(esr),
        voltage,
        tolerance,
    })
}
