use crate::bom::Manufacturer;
use crate::circuit::Part;
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, PinKind};
use crate::resistors::{PowerWatt, ResistanceValues, Tolerance};
use crate::smd::SizeCode;

fn map_size_to_code(size: SizeCode) -> &'static str {
    match size {
        SizeCode::I0075 => "0075",
        SizeCode::I0100 => "0100",
        SizeCode::I0201 => "0201",
        SizeCode::I0402 => "0402",
        SizeCode::I0603 => "0603",
        SizeCode::I0805 => "0805",
        SizeCode::I1206 => "1206",
        SizeCode::I1210 => "1210",
        SizeCode::I1218 => "1218",
        SizeCode::I2010 => "2010",
        SizeCode::I2512 => "2512",
        _ => "NONE",
    }
}

fn map_tolerance_to_code(tol: Tolerance) -> &'static str {
    match tol {
        Tolerance::TenthPercent => "B",
        Tolerance::HalfPercent => "D",
        Tolerance::OnePercent => "F",
        Tolerance::FivePercent => "J",
    }
}

fn make_yageo_rc_l_series_part_number(
    size: SizeCode,
    tol: Tolerance,
    value: ResistanceValues,
) -> String {
    format!(
        "RC{size}{tolerance}R-07{value}L",
        size = map_size_to_code(size),
        tolerance = map_tolerance_to_code(tol),
        value = format!("{:?}", value).replace("Ohm", "")
    )
}

#[test]
fn test_part_number_mapping() {
    // From the spec sheet example
    assert_eq!(
        make_yageo_rc_l_series_part_number(
            SizeCode::I0402,
            Tolerance::FivePercent,
            ResistanceValues::Ohm100K
        ),
        "RC0402JR-07100KL"
    );
}

fn power_rating(size: SizeCode) -> PowerWatt {
    match size {
        SizeCode::I0075 => PowerWatt::Fiftieth,
        SizeCode::I0100 => PowerWatt::ThirtySecond,
        SizeCode::I0201 => PowerWatt::Twentieth,
        SizeCode::I0402 => PowerWatt::Sixteenth,
        SizeCode::I0603 => PowerWatt::Tenth,
        SizeCode::I0805 => PowerWatt::Eighth,
        SizeCode::I1206 => PowerWatt::Quarter,
        SizeCode::I1210 => PowerWatt::Half,
        SizeCode::I1218 => PowerWatt::One,
        SizeCode::I2010 => PowerWatt::ThreeQuarter,
        SizeCode::I2512 => PowerWatt::One,
        _ => panic!("Unsupported size")
    }
}

pub fn make_yageo_rc_l_series_part(
    size: SizeCode,
    tol: Tolerance,
    value: ResistanceValues,
    power: Option<PowerWatt>,
) -> Option<Part> {
    let part_power = power_rating(size);
    if let Some(required_power) = power {
        return None;
    }
    Some(Part {
        label: format!(
            "{:?} {} {} {}",
            value,
            tol.to_string(),
            size.to_string(),
            part_power.to_string()
        )
        .replace("Ohm", ""),
        manufacturer: Manufacturer {
            manufacturer: "Yageo".to_string(),
            part_number: make_yageo_rc_l_series_part_number(size, tol, value),
        },
        description: format!(
            "Yageo RC-L Thin Film Resistor SMD {:?} {} {:?} {}",
            value,
            tol.to_string(),
            size,
            part_power.to_string()
        ),
        comment: "".to_string(),
        pins: vec![EPin::passive(1), EPin::passive(2)],
        suppliers: vec![],
        datasheet: None,
        designator: Designator {
            kind: DesignatorKind::Resistor,
            index: None,
        },
    })
}
