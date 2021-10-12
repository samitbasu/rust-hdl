use rust_hdl_pcb_core::prelude::*;

fn map_erj_to_size(part_number: &str) -> SizeCode {
    match &part_number[3..=4] {
        "2B" => SizeCode::I0402,
        "3B" => SizeCode::I0603,
        "3R" => SizeCode::I0603,
        "6D" => SizeCode::I0805,
        "6B" => SizeCode::I0805,
        "6R" => SizeCode::I0805,
        "8B" => SizeCode::I1206,
        "8R" => SizeCode::I1206,
        _ => panic!("Unsupported size code {}", part_number),
    }
}

fn map_erj_to_power(part_number: &str) -> PowerWatt {
    match &part_number[3..=4] {
        "2B" => PowerWatt::new(166, 1000),
        "3B" => PowerWatt::new(25, 100),
        "3R" => PowerWatt::new(1, 10),
        "6D" => PowerWatt::new(1, 2),
        "6B" => PowerWatt::new(1, 3),
        "6R" => PowerWatt::new(1, 8),
        "8B" => PowerWatt::new(1, 2),
        "8R" => PowerWatt::new(1, 4),
        _ => panic!("Unsupported size code {}", part_number),
    }
}

fn map_era_to_size(part_number: &str) -> SizeCode {
    match &part_number[3..=4] {
        "1A" => SizeCode::I0201,
        "2A" => SizeCode::I0402,
        "3A" => SizeCode::I0603,
        "6A" => SizeCode::I0805,
        "8A" => SizeCode::I1206,
        _ => panic!("Unknown part number size code {}", part_number),
    }
}

fn map_size_code_to_power(size: SizeCode) -> PowerWatt {
    match size {
        SizeCode::I0201 => PowerWatt::new(1, 20),
        SizeCode::I0402 => PowerWatt::new(1, 16),
        SizeCode::I0603 => PowerWatt::new(1, 10),
        SizeCode::I0805 => PowerWatt::new(1, 8),
        SizeCode::I1206 => PowerWatt::new(1, 4),
        _ => panic!("unexpected size code {}", size),
    }
}

fn map_part_number_to_tolerance(part_number: &str) -> f64 {
    match &part_number[6..7] {
        "W" => 0.05,
        "B" => 0.1,
        "C" => 0.25,
        "D" => 0.5,
        "F" => 1.0,
        "G" => 2.0,
        "J" => 5.0,
        _ => panic!("Unexpected tolerance code in part number {}", part_number),
    }
}

fn map_part_number_to_tempco(part_number: &str) -> f64 {
    match &part_number[5..6] {
        "R" => 10.0,
        "P" => 15.0,
        "E" => 25.0,
        "H" => 50.0,
        "K" => 100.0,
        _ => panic!("Unexpected tempco code in part number {}", part_number),
    }
}

fn map_part_number_to_resistance(part_number: &str) -> f64 {
    let part_number = drop_char(&part_number[7..]);
    let sig_fig_num = part_number.len() - 1;
    let sig_figs = &part_number[0..sig_fig_num];
    let exp = &part_number[sig_fig_num..];
    let sig_figs = sig_figs.parse::<f64>().unwrap();
    let exp = exp.parse::<f64>().unwrap();
    sig_figs * (10.0_f64).powf(exp)
}

fn make_panasonic_era_resistor(part_number: &str) -> CircuitNode {
    assert_eq!(&part_number[0..3], "ERA");
    let size = map_era_to_size(part_number);
    let power = map_size_code_to_power(size.clone());
    let tempco = map_part_number_to_tempco(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_ohms = map_part_number_to_resistance(part_number);
    let value = map_resistance_to_string(value_ohms);
    let tempco_string = format!("{} ppm/K", tempco);
    let label = format!("{} {}% {}W", value, tolerance, power);
    let manufacturer = Manufacturer {
        name: "Panasonic".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!(
        "Panasonic ERA Thin Film Resistor SMD {} {} {}",
        size, label, tempco_string
    );
    make_resistor(
        label,
        manufacturer,
        description,
        size,
        value_ohms,
        power,
        tolerance,
        Some(tempco),
        ResistorKind::ThinFilmChip,
    )
}

fn make_panasonic_erj_resistor(part_number: &str) -> CircuitNode {
    assert_eq!(&part_number[0..3], "ERJ");
    let part_number = &part_number.to_string().replace("-", "");
    let size = map_erj_to_size(part_number);
    let power = map_erj_to_power(part_number);
    let tolerance = map_part_number_to_tolerance(part_number);
    let value_ohms = map_resistance_letter_code_to_value(drop_char(&part_number[7..]));
    let value = map_resistance_to_string(value_ohms);
    let label = format!("{} {}% {}W", value, tolerance, power);
    let manufacturer = Manufacturer {
        name: "Panasonic".to_string(),
        part_number: part_number.to_owned(),
    };
    let description = format!("Panasonic ERJ Thick Film Resistor SMD {} {}", size, label);
    make_resistor(
        label,
        manufacturer,
        description,
        size,
        value_ohms,
        power,
        tolerance,
        None,
        ResistorKind::ThickFilmChip,
    )
}

pub fn make_panasonic_resistor(part_number: &str) -> CircuitNode {
    if part_number.starts_with("ERJ") {
        make_panasonic_erj_resistor(part_number)
    } else {
        make_panasonic_era_resistor(part_number)
    }
}
