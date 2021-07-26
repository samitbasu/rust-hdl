use std::fmt::{Display, Formatter};
use crate::bom::Manufacturer;
use crate::smd::SizeCode;
use crate::circuit::Resistor;
use crate::capacitors::make_passive_two_pin;

pub type PowerWatt = num_rational::Rational32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResistorKind {
    ThinFilmChip,
    ThickFilmChip,
    MetalFilm,
    Carbon
}

pub fn map_resistance_to_string(value: f64) -> String {
    if value < 1e3 {
        format!("{:.3}", value).replace(".", "R")
    } else if value < 1e6 {
        format!("{:.3}", value / 1e3).replace(".", "K")
    } else {
        format!("{:.3}", value / 1e6).replace(".", "M")
    }
}

pub fn map_resistance_letter_code_to_value(resistance: &str) -> f64 {
    let mut resistance = resistance.to_owned();
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

pub fn make_chip_resistor(label: String,
                      manufacturer: Manufacturer,
                      description: String,
                      size: SizeCode,
                      value_ohms: f64,
                      power_watt: PowerWatt,
                      tolerance: f64,
                      tempco: Option<f64>,
                      kind: ResistorKind) -> Resistor {
    Resistor {
        details: make_passive_two_pin(label, manufacturer, description, size),
        value_ohms,
        kind,
        power_watt,
        tolerance,
        tempco
    }
}
