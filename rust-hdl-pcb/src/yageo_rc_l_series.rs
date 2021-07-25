use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, PinKind};
use crate::resistors::{PowerMilliWatt, ResistorTolerance, ResistorKind};
use crate::smd::SizeCode;
use crate::circuit::{Resistor, PartDetails};
use std::fs::File;

fn map_part_number_to_size(part: &str) -> SizeCode {
    match &part[2..=5] {
        "0075" => SizeCode::I0075,
        "0100" => SizeCode::I0100,
        "0201" => SizeCode::I0201,
        "0402" => SizeCode::I0402,
        "0603" => SizeCode::I0603,
        "0805" => SizeCode::I0805,
        "1206" => SizeCode::I1206,
        "1210" => SizeCode::I1210,
        "1218" => SizeCode::I1218,
        "2010" => SizeCode::I2010,
        "2512" => SizeCode::I2512,
        _ => panic!("Unsupported part number for Yageo RC-L series {}", part)
    }
}

fn map_part_number_to_tolerance(part: &str) -> ResistorTolerance {
    match &part[6..7] {
        "B" => ResistorTolerance::TenthPercent,
        "D" => ResistorTolerance::HalfPercent,
        "F" => ResistorTolerance::OnePercent,
        "J" => ResistorTolerance::FivePercent,
        _ => panic!("Unsupported tolerance in Yageo FC L resistor part {}", part)
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

fn power_rating(size: SizeCode) -> PowerMilliWatt {
    match size {
        SizeCode::I0075 => PowerMilliWatt::MW20,
        SizeCode::I0100 => PowerMilliWatt::MW31P25,
        SizeCode::I0201 => PowerMilliWatt::MW50,
        SizeCode::I0402 => PowerMilliWatt::MW62P5,
        SizeCode::I0603 => PowerMilliWatt::MW100,
        SizeCode::I0805 => PowerMilliWatt::MW125,
        SizeCode::I1206 => PowerMilliWatt::MW250,
        SizeCode::I1210 => PowerMilliWatt::MW500,
        SizeCode::I1218 => PowerMilliWatt::MW1000,
        SizeCode::I2010 => PowerMilliWatt::MW750,
        SizeCode::I2512 => PowerMilliWatt::MW1000,
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

pub fn make_yageo_rc_l_resistor(part_number: &str) -> Resistor {
    let size = map_part_number_to_size(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_ohms = map_part_number_to_resistance(part_number);
    let value = map_part_number_to_resistance_code(part_number);
    let power = power_rating(size);
    let label = format!("{} {} {}",value,tolerance,power);
    let manufacturer = Manufacturer {
        name: "Yageo".to_string(),
        part_number: part_number.to_owned()
    };
    let description = format!("Yageo RC-L Thick Film Resistor SMD {} {}", size, label);
    Resistor {
        details: PartDetails {
            label,
            manufacturer,
            description,
            comment: "".to_string(),
            pins: vec![EPin::passive(1), EPin::passive(2)],
            suppliers: vec![],
            datasheet: Some(url::Url::parse("https://www.yageo.com/upload/media/product/productsearch/datasheet/rchip/PYu-RC_Group_51_RoHS_L_11.pdf").unwrap()),
            designator: Designator {
                kind: DesignatorKind::Resistor,
                index: None,
            },
            size
        },
        value_ohms,
        kind: ResistorKind::ThickFilmChip,
        power,
        tolerance,
    }
}