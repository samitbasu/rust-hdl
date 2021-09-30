use rust_hdl_pcb_core::capacitors;
use rust_hdl_pcb_core::prelude::*;

fn map_part_number_to_size(part: &str) -> SizeCode {
    match &part[3..4] {
        "1" => SizeCode::I0201,
        "2" => SizeCode::I0402,
        "3" => SizeCode::I0603,
        "4" => SizeCode::I0805,
        "5" => SizeCode::I1206,
        "6" => SizeCode::I1210,
        "8" => SizeCode::I1812,
        "9" => SizeCode::I2220,
        "D" => SizeCode::I3025,
        "E" => SizeCode::I0204,
        _ => panic!("Unknown CGA part size {}", part),
    }
}

fn map_part_number_to_dielectric(part_number: &str) -> DielectricCode {
    (&part_number[6..=8]).parse().unwrap()
}

fn map_part_number_to_voltage(part_number: &str) -> f64 {
    match &part_number[9..=10] {
        "2A" => 100.,
        "0E" => 2.5,
        "0G" => 4.,
        "0J" => 6.3,
        "1A" => 10.,
        "1C" => 16.,
        "1E" => 25.,
        "1V" => 35.,
        "1H" => 50.,
        "1N" => 75.,
        _ => panic!("Unknown working voltage {}!", part_number),
    }
}

fn map_part_number_to_pf(pf: &str) -> f64 {
    map_three_digit_cap_to_pf(&pf[11..])
}

fn map_part_number_to_tolerance(part_number: &str) -> CapacitorTolerance {
    match &part_number[14..15] {
        "C" => CapacitorTolerance::QuarterPF,
        "D" => CapacitorTolerance::HalfPF,
        "J" => CapacitorTolerance::FivePercent,
        "K" => CapacitorTolerance::TenPercent,
        "M" => CapacitorTolerance::TwentyPercent,
        _ => panic!("Unknown part tolerance {}", part_number),
    }
}

pub fn make_tdk_cga_capacitor(part_number: &str) -> CircuitNode {
    let size = map_part_number_to_size(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_pf = map_part_number_to_pf(part_number);
    let value = capacitors::map_pf_to_label(value_pf);
    let dielectric = map_part_number_to_dielectric(part_number);
    let voltage = map_part_number_to_voltage(part_number);
    let label = format!("{} {} {}V {}", value, tolerance, voltage, dielectric);
    let manufacturer = Manufacturer {
        name: "TDK".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!(
        "TDK CGA Series Automotive Grade MLCC Capacitor SMD {} {}",
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

#[cfg(test)]
fn known_parts() -> Vec<String> {
    use std::io::BufRead;
    use std::path::PathBuf;

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test");
    path.push("tdk_cga_parts.txt");
    let list = std::fs::File::open(path).unwrap();
    let lines = std::io::BufReader::new(list).lines();
    lines.into_iter().map(|x| x.unwrap()).collect::<Vec<_>>()
}

#[test]
fn part_number_decodes() {
    for part in &known_parts() {
        println!(
            "Part {} tolerance {} dielectric {} voltage {} size {} value {}",
            part,
            map_part_number_to_tolerance(part),
            map_part_number_to_dielectric(part),
            map_part_number_to_voltage(part),
            map_part_number_to_size(part),
            map_part_number_to_pf(part)
        )
    }
}

#[test]
fn matching_parts() {
    let mut count = 0;
    for part in &known_parts() {
        let tolerance = map_part_number_to_tolerance(part);
        let dielectric = map_part_number_to_dielectric(part);
        let voltage = map_part_number_to_voltage(part);
        let size = map_part_number_to_size(part);
        let pf = map_part_number_to_pf(part);
        if (tolerance == CapacitorTolerance::TwentyPercent)
            && (dielectric == DielectricCode::X7R)
            && (voltage == 100.)
            && (size == SizeCode::I0805)
            && (pf == 100.0 * 1000.0)
        {
            println!(
                "Part {} tolerance {} dielectric {} voltage {} size {} value {}",
                part,
                map_part_number_to_tolerance(part),
                map_part_number_to_dielectric(part),
                map_part_number_to_voltage(part),
                map_part_number_to_size(part),
                map_part_number_to_pf(part)
            );
            count += 1;
        }
    }
    assert_eq!(count, 2);
}
