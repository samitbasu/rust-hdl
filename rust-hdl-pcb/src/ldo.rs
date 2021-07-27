use crate::circuit::{Regulator, PartDetails};
use crate::bom::Manufacturer;
use crate::smd::SizeCode;
use crate::designator::{Designator, DesignatorKind};
use crate::epin::{EPin, PinKind};
use crate::utils::pin_list;

pub fn make_on_semi_ncv33375_regulator(part_number: &str) -> Regulator {
    assert!(part_number.starts_with("NCV33375ST"));
    let voltage = match &part_number[10..=12] {
        "1.8" => 1.8,
        "2.5" => 2.5,
        "3.0" => 3.0,
        "3.3" => 3.3,
        "5.0" => 5.0,
        _ => panic!("Unexpected voltage in part {}", part_number),
    };
    Regulator {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer { name: "ON Semiconductor".to_string(), part_number: part_number.into() },
            description: "300mA LDO Automotive, 13V input range".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(vec![
                EPin {
                    kind: PinKind::PowerSink,
                    name: "VIN".into(),
                },
                EPin {
                    kind: PinKind::Input,
                    name: "ON/OFF".into(),
                },
                EPin {
                    kind: PinKind::PowerSource,
                    name: "VOUT".into(),
                },
                EPin {
                    kind: PinKind::PowerReturn,
                    name: "GND".into(),
                }
            ]),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::VoltageRegulator, index: None },
            size: SizeCode::SOT223,
        },
        input_min_voltage: 0.260 + voltage,
        input_max_voltage: 13.0,
        output_nominal_voltage: voltage,
        output_max_current_ma: 300.0,
    }
}

pub fn make_mcp_1799_regulator(part_number: &str) -> Regulator {
    assert!(part_number.starts_with("MCP1799"));
    let voltage = match &part_number[9..=10] {
        "33" => 3.3,
        "50" => 5.0,
        _ => panic!("unwknown part number")
    };
    Regulator {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer { name: "Microchip".to_string(), part_number: part_number.into() },
            description: "80mA LDO Automotive, 45V input range".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(vec![
                EPin {
                    kind: PinKind::PowerSink,
                    name: "VIN".into(),
                },
                EPin {
                    kind: PinKind::PowerReturn,
                    name: "GND_1".into(),
                },
                EPin {
                    kind: PinKind::PowerSource,
                    name: "VOUT".into(),
                },
                EPin {
                    kind: PinKind::PowerReturn,
                    name: "GND_2".into(),
                }
            ]),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::VoltageRegulator, index: None },
            size: SizeCode::SOT223,
        },
        input_min_voltage: 0.3 + voltage,
        input_max_voltage: 45.0,
        output_nominal_voltage: voltage,
        output_max_current_ma: 80.0,
    }
}

pub fn make_ti_tps_7b84_regulator(part_number: &str) -> Regulator {
    assert!(part_number.starts_with("TPS7B84"));
    let voltage = match &part_number[7..=8] {
        "33" => 3.3,
        "50" => 5.0,
        _ => panic!("unknown part number")
    };
    Regulator {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer { name: "TI".to_string(), part_number: part_number.into() },
            description: "150mA LDO, 40V input range".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(vec![
                EPin {
                    kind: PinKind::PowerSink,
                    name: "IN".into(),
                },
                EPin {
                    kind: PinKind::Input,
                    name: "EN".into(),
                },
                EPin {
                    kind: PinKind::PowerSource,
                    name: "OUT".into(),
                },
                EPin {
                    kind: PinKind::PowerReturn,
                    name: "GND".into(),
                }
            ]),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::VoltageRegulator, index: None },
            size: SizeCode::SOT223,
        },
        input_min_voltage: 0.3 + voltage,
        input_max_voltage: 42.0,
        output_nominal_voltage: voltage,
        output_max_current_ma: 150.0,
    }
}

pub fn make_zldo1117g_regulator(part_number: &str) -> Regulator {
    assert!(part_number.starts_with("ZLDO1117G"));
    assert!(part_number.ends_with("TA"));
    let voltage = match &part_number[9..=10] {
        "12" => 1.2,
        "15" => 1.5,
        "18" => 1.8,
        "25" => 2.5,
        "33" => 3.3,
        "50" => 5.0,
        _ => panic!("Unrecognized part number {}", part_number)
    };
    Regulator {
        details: PartDetails {
            label: part_number.to_string(),
            manufacturer: Manufacturer { name: "Diodes".to_string(), part_number: part_number.into() },
            description: "1A LDO, 18V input range".to_string(),
            comment: "".to_string(),
            hide_pin_designators: false,
            pins: pin_list(vec![
                EPin {
                    kind: PinKind::PowerReturn,
                    name: "GND".into(),
                },
                EPin {
                    kind: PinKind::PowerSource,
                    name: "Vout_1".into(),
                },
                EPin {
                    kind: PinKind::PowerSink,
                    name: "Vin".into(),
                },
                EPin {
                    kind: PinKind::PowerSource,
                    name: "Vout_2".into(),
                }
            ]),
            suppliers: vec![],
            designator: Designator { kind: DesignatorKind::VoltageRegulator, index: None },
            size: SizeCode::SOT223,
        },
        input_min_voltage: 2.7 + voltage,
        input_max_voltage: 18.0,
        output_nominal_voltage: voltage,
        output_max_current_ma: 1000.0,
    }
}