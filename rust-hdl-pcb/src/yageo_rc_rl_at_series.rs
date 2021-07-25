use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, PinKind};
use crate::resistors::{PowerWatt, ResistorTolerance, ResistorKind, ResistorTempco};
use crate::smd::SizeCode;
use crate::circuit::{Resistor, PartDetails};
use std::fs::File;
use url::Url;
use std::fmt::{Display, Formatter};

fn map_part_number_to_size(part: &str) -> SizeCode {
    (&part[2..=5]).parse().unwrap()
}

fn map_part_number_to_tolerance(part: &str) -> ResistorTolerance {
    match &part[6..7] {
        "B" => ResistorTolerance::TenthPercent,
        "D" => ResistorTolerance::HalfPercent,
        "F" => ResistorTolerance::OnePercent,
        "G" => ResistorTolerance::TwoPercent,
        "J" => ResistorTolerance::FivePercent,
        _ => panic!("Unsupported tolerance in Yageo FC L resistor part {}", part)
    }
}

fn map_part_number_to_tempco(part: &str) -> Option<ResistorTempco> {
    match &part[8..9] {
        "C" => Some(ResistorTempco::Ppm15degC),
        "D" => Some(ResistorTempco::Ppm25degC),
        "E" => Some(ResistorTempco::Ppm50degC),
        _ => None,
    }
}

fn drop_char(txt: &str) -> &str {
    let len = txt.len();
    &txt[..(len-1)]
}

fn map_part_number_to_resistance_code(part: &str) -> &str {
    drop_char(&part[11..])
}

fn map_part_number_to_resistance(part: &str) -> f64 {
    let mut resistance = map_part_number_to_resistance_code(part).to_owned();
    let mut multiplier = 1.0;
    if resistance.contains("K") {
        multiplier = 1.0e3;
        resistance = resistance.replace("K", ".");
    }
    if resistance.contains("M") {
        multiplier = 1.0e6;
        resistance = resistance.replace("M", ".");
    }
    if resistance.contains("R") {
        multiplier = 1.0;
        resistance = resistance.replace("R", ".");
    }
    resistance.parse::<f64>().unwrap() * multiplier
}

fn power_rating(size: SizeCode) -> PowerWatt {
    match size {
        SizeCode::I0075 => PowerWatt::new(1, 50),
        SizeCode::I0100 => PowerWatt::new(1, 32),
        SizeCode::I0201 => PowerWatt::new(1,20),
        SizeCode::I0402 => PowerWatt::new(1, 16),
        SizeCode::I0603 => PowerWatt::new(1,10),
        SizeCode::I0805 => PowerWatt::new(1,8),
        SizeCode::I1206 => PowerWatt::new(1,4),
        SizeCode::I1210 => PowerWatt::new(1,2),
        SizeCode::I1218 => PowerWatt::new(1, 1),
        SizeCode::I2010 => PowerWatt::new(3, 4),
        SizeCode::I2512 => PowerWatt::new(1, 1),
        _ => panic!("Unsupported size")
    }
}

#[test]
fn test_part_number_mapping() {
    // From the spec sheet example
    let part_number = "RC0402JR-07100KL";
    assert_eq!(map_part_number_to_size(part_number), SizeCode::I0402);
    assert_eq!(map_part_number_to_tolerance(part_number), ResistorTolerance::FivePercent);
    assert_eq!(map_part_number_to_resistance(part_number), 100e3);
}

#[test]
fn test_part_number_parses() {
    use std::path::PathBuf;
    use std::io::BufRead;

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
        println!("Part {} -> {} {} {}", part_number, size, tolerance, resistance);
    }
}

#[derive(Copy, Clone, PartialEq)]
enum YageoSeries {
    RC,
    RL,
    AT
}

impl Display for YageoSeries {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            YageoSeries::RC => "RC".fmt(f),
            YageoSeries::RL => "RL".fmt(f),
            YageoSeries::AT => "AT".fmt(f),
        }
    }
}

fn datasheet(series: YageoSeries) -> Url {
    match series {
        YageoSeries::RC => url::Url::parse("https://www.yageo.com/upload/media/product/productsearch/datasheet/rchip/PYu-RC_Group_51_RoHS_L_11.pdf").unwrap(),
        YageoSeries::RL => url::Url::parse("https://www.yageo.com/upload/media/product/productsearch/datasheet/rchip/PYu-RL_Group_521_RoHS_L_2.pdf").unwrap(),
        YageoSeries::AT => url::Url::parse("https://www.yageo.com/upload/media/product/productsearch/datasheet/rchip/PYu-AT_51_RoHS_L_5.pdf").unwrap()
    }
}

fn make_yageo(part_number: &str, series: YageoSeries) -> Resistor {
    let size = map_part_number_to_size(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_ohms = map_part_number_to_resistance(part_number);
    let value = map_part_number_to_resistance_code(part_number);
    let power = power_rating(size);
    let tempco = map_part_number_to_tempco(part_number);
    let tempco_string = if let Some(k) = tempco {
        k.to_string()
    } else {
        "".to_owned()
    };
    let label = format!("{} {} {}W",value,tolerance,power);
    let manufacturer = Manufacturer {
        name: "Yageo".to_string(),
        part_number: part_number.to_owned()
    };
    let description = format!("Yageo {} Thick Film Resistor SMD {} {} {}", series, size, label, tempco_string);
    Resistor {
        details: PartDetails {
            label,
            manufacturer,
            description,
            comment: "".to_string(),
            pins: vec![EPin::passive(1), EPin::passive(2)],
            suppliers: vec![],
            datasheet: Some(datasheet(series)),
            designator: Designator {
                kind: DesignatorKind::Resistor,
                index: None,
            },
            size
        },
        value_ohms,
        kind: ResistorKind::ThickFilmChip,
        power_watt: power,
        tolerance,
        tempco: map_part_number_to_tempco(part_number),
    }
}

pub fn make_yageo_series_resistor(part_number: &str) -> Resistor {
    match &part_number[0..2] {
        "RC" => make_yageo(part_number, YageoSeries::RC),
        "RL" => make_yageo(part_number, YageoSeries::RL),
        "AT" => make_yageo(part_number, YageoSeries::AT),
        _ => panic!("unrecognized Yageo series {}", part_number)
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