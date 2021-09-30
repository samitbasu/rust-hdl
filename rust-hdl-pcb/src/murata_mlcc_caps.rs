use rust_hdl_pcb_core::prelude::*;

fn make_murata_grt_188r61h_capacitor(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("GRT188R61H"));
    let size = SizeCode::I0603;
    let tolerance = CapacitorTolerance::TenPercent;
    let value_pf = map_three_digit_cap_to_pf(&part_number[10..=12]);
    let dielectric = DielectricCode::X5R;
    let voltage = 50.0;
    let value = map_pf_to_label(value_pf);
    let label = format!("{} {} {}V {}", value, tolerance, voltage, dielectric);
    let manufacturer = Manufacturer {
        name: "muRata".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!(
        "muRata AEC-Q200 GRT188 Series MLCC Capacitor SMD {} {}",
        size, label
    );
    make_mlcc(
        label,
        manufacturer,
        description,
        size,
        value_pf,
        dielectric,
        voltage,
        tolerance,
    )
}

fn make_murata_grm21br61c_capacitor(part_number: &str) -> CircuitNode {
    assert!(part_number.starts_with("GRM21BR61C"));
    let size = SizeCode::I0805;
    let tolerance = CapacitorTolerance::TwentyPercent;
    let value_pf = map_three_digit_cap_to_pf(&part_number[10..=12]);
    let dielectric = DielectricCode::X5R;
    let voltage = 16.0;
    let value = map_pf_to_label(value_pf);
    let label = format!("{} {} {}V {}", value, tolerance, voltage, dielectric);
    let manufacturer = Manufacturer {
        name: "muRata".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!(
        "muRata General GRM21B Series MLCC Capacitor SMD {} {}",
        size, label
    );
    make_mlcc(
        label,
        manufacturer,
        description,
        size,
        value_pf,
        dielectric,
        voltage,
        tolerance,
    )
}

pub fn make_murata_capacitor(part_number: &str) -> CircuitNode {
    if part_number.starts_with("GRM21BR61C") {
        make_murata_grm21br61c_capacitor(part_number)
    } else if part_number.starts_with("GRT188R61H") {
        make_murata_grt_188r61h_capacitor(part_number)
    } else {
        unimplemented!()
    }
}
