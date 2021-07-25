use crate::smd::SizeCode;
use crate::capacitors::{DielectricCode, WorkingVoltage, CapacitorTolerance, CapacitorKind};
use crate::circuit::{Capacitor, PartDetails};
use crate::bom::Manufacturer;
use crate::epin::EPin;
use crate::designator::{Designator, DesignatorKind};

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
        _ => panic!("Unknown CGA part size {}", part)
    }
}

fn map_part_number_to_dielectric(part_number: &str) -> DielectricCode {
    match &part_number[6..=8] {
        "X5R" => DielectricCode::X5R,
        "X7R" => DielectricCode::X7R,
        "C0G" => DielectricCode::C0G,
        "NP0" => DielectricCode::C0G,
        "X7T" => DielectricCode::X7T,
        _ => panic!("Unknown dielectric code {}", part_number)
    }
}

fn map_part_number_to_voltage(part_number: &str) -> WorkingVoltage {
    match &part_number[9..=10] {
        "2A" => WorkingVoltage::V100,
        "0E" => WorkingVoltage::V2V5,
        "0G" => WorkingVoltage::V4,
        "0J" => WorkingVoltage::V6V3,
        "1A" => WorkingVoltage::V10,
        "1C" => WorkingVoltage::V16,
        "1E" => WorkingVoltage::V25,
        "1V" => WorkingVoltage::V35,
        "1H" => WorkingVoltage::V50,
        "1N" => WorkingVoltage::V75,
        _ => panic!("Unknown working voltage {}!", part_number)
    }
}

fn map_part_number_to_pf(pf: &str) -> f64 {
    if &pf[12..13] == "R" {
        let pf_ones = &pf[11..12].parse::<f64>().unwrap();
        let pf_tenths = &pf[13..14].parse::<f64>().unwrap();
        return pf_ones + pf_tenths * 0.1;
    } else {
        let pf_tens = &pf[11..12].parse::<f64>().unwrap();
        let pf_ones = &pf[12..13].parse::<f64>().unwrap();
        let pf_exp = &pf[13..14].parse::<f64>().unwrap();
        return (pf_tens * 10.0 + pf_ones) * 10.0_f64.powf(*pf_exp);
    }
}

fn map_part_number_to_tolerance(part_number: &str) -> CapacitorTolerance {
    match &part_number[14..15] {
        "C" => CapacitorTolerance::QuarterPF,
        "D" => CapacitorTolerance::HalfPF,
        "J" => CapacitorTolerance::FivePercent,
        "K" => CapacitorTolerance::TenPercent,
        "M" => CapacitorTolerance::TwentyPercent,
        _ => panic!("Unknown part tolerance {}", part_number)
    }
}

fn map_pf_to_label(value: f64) -> String {
    if value < 1e3 {
        // pF case
        format!("{:.1} pF", value)
    } else if value < 1e6 {
        // nF case
        format!("{:.1} nF", value / 1e3)
    } else if value < 1e9 {
        // uF case
        format!("{:.1} uF", value / 1e6)
    } else {
        // mF case??
        format!("{:.1} mF", value / 1e9)
    }
}


pub fn make_tdk_cga_capacitor(part_number: &str) -> Capacitor {
    let size = map_part_number_to_size(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_pf = map_part_number_to_pf(part_number);
    let value = map_pf_to_label(value_pf);
    let dielectric = map_part_number_to_dielectric(part_number);
    let voltage = map_part_number_to_voltage(part_number);
    let label = format!("{} {} {} {}", value, tolerance, voltage, dielectric);
    let manufacturer = Manufacturer {
        name: "TDK".to_string(),
        part_number: part_number.to_owned()
    };
    let description = format!("TDK CGA Series Automotive Grade MLCC Capacitor SMD {} {}", size, label);
    Capacitor {
        details: PartDetails {
            label,
            manufacturer,
            description,
            comment: "".to_string(),
            pins: vec![EPin::passive(1), EPin::passive(2)],
            suppliers: vec![],
            datasheet: None,
            designator: Designator {
                kind: DesignatorKind::Capacitor,
                index: None,
            },
            size,
        },
        value_pf,
        kind: CapacitorKind::MultiLayerChip,
        voltage,
        dielectric,
        tolerance,
    }
}


#[cfg(test)]
fn known_parts() -> Vec<String> {
    use std::path::PathBuf;
    use std::io::BufRead;

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
        println!("Part {} tolerance {} dielectric {} voltage {} size {} value {}", part,
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
        if (tolerance == CapacitorTolerance::TwentyPercent) &&
            (dielectric == DielectricCode::X7R) &&
            (voltage == WorkingVoltage::V100) &&
            (size == SizeCode::I0805) &&
            (pf == 100.0 * 1000.0) {
            println!("Part {} tolerance {} dielectric {} voltage {} size {} value {}", part,
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