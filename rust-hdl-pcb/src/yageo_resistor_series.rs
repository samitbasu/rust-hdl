use std::fmt::{Display, Formatter};
use std::fs::File;

use rust_hdl_pcb_core::prelude::*;

fn map_part_number_to_size(part: &str) -> SizeCode {
    (&part[2..=5]).parse().unwrap()
}

fn map_part_number_to_tolerance(part: &str) -> f64 {
    match &part[6..7] {
        "B" => 0.1,
        "D" => 0.5,
        "F" => 1.0,
        "G" => 2.0,
        "J" => 5.0,
        _ => panic!("Unsupported tolerance in Yageo FC L resistor part {}", part),
    }
}

fn map_part_number_to_tempco(part: &str) -> Option<f64> {
    match &part[8..9] {
        "C" => Some(15.),
        "D" => Some(25.),
        "E" => Some(50.),
        _ => None,
    }
}

fn map_part_number_to_resistance_code(part: &str) -> &str {
    drop_char(&part[11..])
}

fn map_part_number_to_resistance(part: &str) -> f64 {
    let resistance = map_part_number_to_resistance_code(part).to_owned();
    map_resistance_letter_code_to_value(&resistance)
}

fn power_rating(size: SizeCode) -> PowerWatt {
    match size {
        SizeCode::I0075 => PowerWatt::new(1, 50),
        SizeCode::I0100 => PowerWatt::new(1, 32),
        SizeCode::I0201 => PowerWatt::new(1, 20),
        SizeCode::I0402 => PowerWatt::new(1, 16),
        SizeCode::I0603 => PowerWatt::new(1, 10),
        SizeCode::I0805 => PowerWatt::new(1, 8),
        SizeCode::I1206 => PowerWatt::new(1, 4),
        SizeCode::I1210 => PowerWatt::new(1, 2),
        SizeCode::I1218 => PowerWatt::new(1, 1),
        SizeCode::I2010 => PowerWatt::new(3, 4),
        SizeCode::I2512 => PowerWatt::new(1, 1),
        _ => panic!("Unsupported size"),
    }
}

#[test]
fn test_part_number_mapping() {
    // From the spec sheet example
    let part_number = "RC0402JR-07100KL";
    assert_eq!(map_part_number_to_size(part_number), SizeCode::I0402);
    assert_eq!(map_part_number_to_tolerance(part_number), 5.0);
    assert_eq!(map_part_number_to_resistance(part_number), 100e3);
}

#[test]
fn test_part_number_parses() {
    use std::io::BufRead;
    use std::path::PathBuf;

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test");
    path.push("yageo_rc_l_parts.txt");
    let list = File::open(path).unwrap();
    let lines = std::io::BufReader::new(list).lines();
    for part in lines {
        let part_number = part.unwrap();
        let size = map_part_number_to_size(&part_number);
        let tolerance = map_part_number_to_tolerance(&part_number);
        let resistance = map_part_number_to_resistance(&part_number);
        println!(
            "Part {} -> {} {} {}",
            part_number, size, tolerance, resistance
        );
    }
}

#[derive(Copy, Clone, PartialEq)]
enum YageoSeries {
    RC,
    RL,
    AT,
    FMP50,
    FMP100,
    FMP200,
}

impl Display for YageoSeries {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            YageoSeries::RC => "RC".fmt(f),
            YageoSeries::RL => "RL".fmt(f),
            YageoSeries::AT => "AT".fmt(f),
            YageoSeries::FMP50 => "FMP-50".fmt(f),
            YageoSeries::FMP100 => "FMP100".fmt(f),
            YageoSeries::FMP200 => "FMP200".fmt(f),
        }
    }
}

fn map_pth_series_to_size(series: YageoSeries) -> SizeCode {
    match series {
        YageoSeries::FMP50 => SizeCode::PTHResistor(PTHResistor {
            body_length: TolerancedDim {
                nominal_mm: 3.4,
                tolerance_mm: 0.3,
            },
            body_diameter: TolerancedDim {
                nominal_mm: 1.9,
                tolerance_mm: 0.2,
            },
            lead_length: TolerancedDim {
                nominal_mm: 28.0,
                tolerance_mm: 2.0,
            },
            lead_diameter: TolerancedDim {
                nominal_mm: 0.45,
                tolerance_mm: 0.05,
            },
        }),
        YageoSeries::FMP100 => SizeCode::PTHResistor(PTHResistor {
            body_length: TolerancedDim {
                nominal_mm: 6.3,
                tolerance_mm: 0.5,
            },
            body_diameter: TolerancedDim {
                nominal_mm: 2.4,
                tolerance_mm: 0.2,
            },
            lead_length: TolerancedDim {
                nominal_mm: 28.0,
                tolerance_mm: 2.0,
            },
            lead_diameter: TolerancedDim {
                nominal_mm: 0.55,
                tolerance_mm: 0.05,
            },
        }),
        YageoSeries::FMP200 => SizeCode::PTHResistor(PTHResistor {
            body_length: TolerancedDim {
                nominal_mm: 9.0,
                tolerance_mm: 0.5,
            },
            body_diameter: TolerancedDim {
                nominal_mm: 3.9,
                tolerance_mm: 0.3,
            },
            lead_length: TolerancedDim {
                nominal_mm: 26.0,
                tolerance_mm: 2.0,
            },
            lead_diameter: TolerancedDim {
                nominal_mm: 0.55,
                tolerance_mm: 0.05,
            },
        }),
        _ => unimplemented!(),
    }
}

fn map_pth_series_to_power(series: YageoSeries) -> PowerWatt {
    match series {
        YageoSeries::FMP50 => PowerWatt::new(1, 2),
        YageoSeries::FMP100 => PowerWatt::new(1, 1),
        YageoSeries::FMP200 => PowerWatt::new(2, 1),
        _ => unimplemented!(),
    }
}

fn make_yageo_pth(part_number: &str, series: YageoSeries) -> CircuitNode {
    let size = map_pth_series_to_size(series);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_ohms = map_resistance_letter_code_to_value(&part_number[12..]);
    let value = map_resistance_to_string(value_ohms);
    let power = map_pth_series_to_power(series);
    let label = format!("{} {}% {}W", value, tolerance, power);
    let manufacturer = Manufacturer {
        name: "Yageo".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!("Yageo {} Metal Film PTH {} {}", series, size, label);
    make_resistor(
        label,
        manufacturer,
        description,
        size,
        value_ohms,
        power,
        tolerance,
        None,
        ResistorKind::MetalFilm,
    )
}

fn make_yageo_chip(part_number: &str, series: YageoSeries) -> CircuitNode {
    let size = map_part_number_to_size(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_ohms = map_part_number_to_resistance(part_number);
    let value = map_part_number_to_resistance_code(part_number);
    let power = power_rating(size.clone());
    let tempco = map_part_number_to_tempco(part_number);
    let tempco_string = if let Some(k) = tempco {
        format!("{} ppm/C", k)
    } else {
        "".to_owned()
    };
    let label = format!("{} {}% {}W", value, tolerance, power);
    let manufacturer = Manufacturer {
        name: "Yageo".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!(
        "Yageo {} Thick Film Resistor SMD {} {} {}",
        series, size, label, tempco_string
    );
    make_resistor(
        label,
        manufacturer,
        description,
        size,
        value_ohms,
        power,
        tolerance,
        tempco,
        ResistorKind::ThickFilmChip,
    )
}

pub fn make_yageo_series_resistor(part_number: &str) -> CircuitNode {
    match &part_number[0..2] {
        "RC" => make_yageo_chip(part_number, YageoSeries::RC),
        "RL" => make_yageo_chip(part_number, YageoSeries::RL),
        "AT" => make_yageo_chip(part_number, YageoSeries::AT),
        _ => match &part_number[0..6] {
            "FMP-50" => make_yageo_pth(part_number, YageoSeries::FMP50),
            "FMP100" => make_yageo_pth(part_number, YageoSeries::FMP100),
            "FMP200" => make_yageo_pth(part_number, YageoSeries::FMP200),
            _ => panic!("unrecognized Yageo series {}", part_number),
        },
    }
}

#[test]
fn test_ohm_decodes() {
    assert_eq!(map_part_number_to_resistance("RL0603FR-070R56L"), 0.56);
    assert_eq!(map_part_number_to_resistance("RL0603FR-070R001L"), 0.001);
    assert_eq!(map_part_number_to_resistance("RL0603FR-071R76L"), 1.76);
    assert_eq!(map_part_number_to_resistance("RL0603FR-0715R8L"), 15.8);
    assert_eq!(map_part_number_to_resistance("RL0603FR-07154RL"), 154.);
    assert_eq!(map_part_number_to_resistance("RL0603FR-071K38L"), 1.38e3);
    assert_eq!(map_part_number_to_resistance("RL0603FR-071M38L"), 1.38e6);
}
