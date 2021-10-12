use rust_hdl_pcb_core::prelude::*;

fn map_part_number_to_size(part: &str) -> SizeCode {
    (&part[0..=3]).parse().unwrap()
}

fn map_part_number_to_voltage(part: &str) -> f64 {
    match &part[4..5] {
        "4" => 4.0,
        "6" => 6.3,
        "Z" => 10.0,
        "Y" => 16.0,
        "3" => 25.0,
        "5" => 50.0,
        "1" => 100.0,
        "2" => 200.0,
        "7" => 500.0,
        _ => panic!("No working voltage for {}", part),
    }
}

fn map_part_number_to_dielectric(part: &str) -> DielectricCode {
    match &part[5..6] {
        "C" => DielectricCode::X7R,
        _ => panic!("Unknown dielectric code for AVX {}", part),
    }
}

fn map_part_number_to_pf(part: &str) -> f64 {
    map_three_digit_cap_to_pf(&part[6..9])
}

fn map_part_number_to_tolerance(part: &str) -> CapacitorTolerance {
    match &part[9..10] {
        "J" => CapacitorTolerance::FivePercent,
        "K" => CapacitorTolerance::TenPercent,
        "M" => CapacitorTolerance::TwentyPercent,
        _ => panic!("Unknon capacitor tolerance indicator {}", part),
    }
}

pub fn make_avx_capacitor(part_number: &str) -> CircuitNode {
    let size = map_part_number_to_size(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_pf = map_part_number_to_pf(part_number);
    let value = map_pf_to_label(value_pf);
    let dielectric = map_part_number_to_dielectric(part_number);
    let voltage = map_part_number_to_voltage(part_number);
    let label = format!("{} {} {}V {}", value, tolerance, voltage, dielectric);
    let manufacturer = Manufacturer {
        name: "AVX".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!("AVX X7R Series MLCC Capacitor SMD {} {}", size, label);
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
