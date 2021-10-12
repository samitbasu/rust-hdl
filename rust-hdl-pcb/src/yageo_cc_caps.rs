use rust_hdl_pcb_core::prelude::*;

fn map_part_number_to_size(part: &str) -> SizeCode {
    (&part[2..=5]).parse().unwrap()
}

fn map_part_number_to_voltage(part: &str) -> f64 {
    match &part[11..12] {
        "4" => 4.0,
        "5" => 6.3,
        "6" => 10.0,
        "7" => 16.0,
        "8" => 25.0,
        "9" => 50.0,
        _ => panic!("No working voltage for {}", part),
    }
}

fn map_part_number_to_dielectric(part: &str) -> DielectricCode {
    // TODO - handle all dielectric codes...
    (&part[8..=10]).parse().unwrap()
}

fn map_part_number_to_pf(part: &str) -> f64 {
    map_three_digit_cap_to_pf(&part[14..=16])
}

fn map_part_number_to_tolerance(part: &str) -> CapacitorTolerance {
    match &part[6..7] {
        "K" => CapacitorTolerance::TenPercent,
        "M" => CapacitorTolerance::TwentyPercent,
        _ => panic!("Unknown capacitor tolerance indicator {}", part),
    }
}

pub fn make_yageo_cc_series_cap(part_number: &str) -> CircuitNode {
    assert_eq!(&part_number[0..2], "CC");
    let size = map_part_number_to_size(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_pf = map_part_number_to_pf(part_number);
    let value = map_pf_to_label(value_pf);
    let dielectric = map_part_number_to_dielectric(part_number);
    let voltage = map_part_number_to_voltage(part_number);
    let label = format!("{} {} {}V {}", value, tolerance, voltage, dielectric);
    let manufacturer = Manufacturer {
        name: "Yageo".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!(
        "Yageo commercial CC Series MLCC Capacitor SMD {} {}",
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
