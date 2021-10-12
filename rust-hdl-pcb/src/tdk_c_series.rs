use rust_hdl_pcb_core::prelude::*;

fn map_part_number_to_size(part: &str) -> SizeCode {
    match &part[0..=4] {
        "C0603" => SizeCode::I0201,
        "C1005" => SizeCode::I0402,
        "C1608" => SizeCode::I0603,
        "C2012" => SizeCode::I0805,
        "C3216" => SizeCode::I1206,
        "C3225" => SizeCode::I1210,
        "C4532" => SizeCode::I1812,
        "C5750" => SizeCode::I2220,
        _ => panic!("Unrecognized size code {}", part),
    }
}

fn map_part_number_to_voltage(part: &str) -> f64 {
    match &part[8..=9] {
        "0G" => 4.0,
        "0J" => 6.3,
        "1A" => 10.0,
        "1C" => 16.0,
        "1E" => 25.0,
        "1V" => 35.0,
        "1H" => 50.0,
        "1N" => 75.0,
        _ => panic!("No working voltage for {}", part),
    }
}

fn map_part_number_to_dielectric(part: &str) -> DielectricCode {
    // TODO - handle all dielectric codes...
    (&part[5..=7]).parse().unwrap()
}

fn map_part_number_to_pf(part: &str) -> f64 {
    map_three_digit_cap_to_pf(&part[10..=12])
}

fn map_part_number_to_tolerance(part: &str) -> CapacitorTolerance {
    match &part[13..14] {
        "B" => CapacitorTolerance::TenthPF,
        "C" => CapacitorTolerance::QuarterPF,
        "D" => CapacitorTolerance::HalfPF,
        "F" => CapacitorTolerance::OnePercent,
        "G" => CapacitorTolerance::TwoPercent,
        "J" => CapacitorTolerance::FivePercent,
        "K" => CapacitorTolerance::TenPercent,
        "M" => CapacitorTolerance::TwentyPercent,
        _ => panic!("Unknown capacitor tolerance indicator {}", part),
    }
}

pub fn make_tdk_c_series_capacitor(part_number: &str) -> CircuitNode {
    assert_eq!(&part_number[0..1], "C");
    let size = map_part_number_to_size(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_pf = map_part_number_to_pf(part_number);
    let value = map_pf_to_label(value_pf);
    let dielectric = map_part_number_to_dielectric(part_number);
    let voltage = map_part_number_to_voltage(part_number);
    let label = format!("{} {} {}V {}", value, tolerance, voltage, dielectric);
    let manufacturer = Manufacturer {
        name: "TDK".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!(
        "TDK Commercial C Series MLCC Capacitor SMD {} {}",
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
