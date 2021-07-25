use crate::capacitors::CapacitanceValues;
use crate::circuit::Part;
use crate::bom::Manufacturer;
use crate::designator::{Designator, DesignatorKind};

pub fn make_murata_grt_188r61h_part(value: CapacitanceValues) -> Part {
    Part {
        label: "".to_string(),
        manufacturer: Manufacturer { manufacturer: "".to_string(), part_number: "".to_string() },
        description: "".to_string(),
        comment: "".to_string(),
        pins: vec![],
        suppliers: vec![],
        datasheet: None,
        designator: Designator { kind: DesignatorKind::Resistor, index: None },
    }
}
