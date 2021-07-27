use crate::circuit::{Capacitor, PartDetails};
use crate::capacitors::{map_three_digit_cap_to_pf, CapacitorTolerance, CapacitorKind, map_pf_to_label};
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};
use crate::smd::SizeCode;
use crate::epin::EPin;
use crate::utils::pin_list;


fn map_part_number_to_pf(part_number: &str) -> f64 {
    let pf_val = &part_number[5..8];
    map_three_digit_cap_to_pf(pf_val)
}

fn map_part_number_to_tolerance(part_number: &str) -> CapacitorTolerance {
    match &part_number[8..9] {
        "K" => CapacitorTolerance::TenPercent,
        "M" => CapacitorTolerance::TwentyPercent,
        _ => panic!("Unexpected part number {}", part_number),
    }
}

fn map_part_number_to_voltage(part_number: &str) -> f64 {
    match &part_number[9..12] {
        "2R5" => 2.5,
        "003" => 3.,
        "004" => 4.,
        "006" => 6.,
        "010" => 10.,
        "016" => 16.,
        "020" => 20.,
        "025" => 25.,
        "035" => 35.,
        "050" => 50.,
        _ => panic!("Unexpected voltage in part number {}", part_number)
    }
}

pub fn make_kemet_t491_capacitor(part_number: &str) -> Capacitor {
    assert!(part_number.starts_with("T491A"));
    let voltage = map_part_number_to_voltage(part_number);
    let value_pf = map_part_number_to_pf(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value = map_pf_to_label(value_pf);
    let label = format!("{} {} {}V Ta", value, tolerance, voltage);
    let size = SizeCode::I1206;
    let description = format!("Kemet T491 Series MnO2 Tantalum Capacitor SMD {} {}",
        size, label);
    Capacitor {
        details: PartDetails {
            label,
            manufacturer: Manufacturer { name: "Kemet".to_string(),
                part_number: part_number.to_owned() },
            description,
            comment: "".to_string(),
            pins: pin_list(vec![EPin::passive_pos(), EPin::passive_neg()]),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::Capacitor, index: None },
            size,
        },
        value_pf,
        kind: CapacitorKind::Tantalum,
        voltage,
        tolerance,
    }
}