use std::fmt::{Display, Formatter, Pointer};

pub fn map_three_digit_cap_to_pf(pf: &str) -> f64 {
    return if &pf[1..2] == "R" {
        let pf_ones = &pf[0..1].parse::<f64>().unwrap();
        let pf_tenths = &pf[2..3].parse::<f64>().unwrap();
        pf_ones + pf_tenths * 0.1
    } else {
        let pf_tens = &pf[0..1].parse::<f64>().unwrap();
        let pf_ones = &pf[1..2].parse::<f64>().unwrap();
        let pf_exp = &pf[2..3].parse::<f64>().unwrap();
        (pf_tens * 10.0 + pf_ones) * 10.0_f64.powf(*pf_exp)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DielectricCode {
    X5R,
    X7R,
    C0G,
    X7T,
}

impl Display for DielectricCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DielectricCode::X5R => "X5R",
            DielectricCode::X7R => "X7R",
            DielectricCode::C0G => "C0G",
            DielectricCode::X7T => "X7T",
        }.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CapacitorKind {
    MultiLayerChip(DielectricCode),
    Tantalum,
    AluminumPolyLowESR,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CapacitorTolerance {
    QuarterPF,
    HalfPF,
    FivePercent,
    TenPercent,
    TwentyPercent,
}

impl Display for CapacitorTolerance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CapacitorTolerance::QuarterPF => "0.25PF",
            CapacitorTolerance::HalfPF => "0.5PF",
            CapacitorTolerance::FivePercent => "5%",
            CapacitorTolerance::TenPercent => "10%",
            CapacitorTolerance::TwentyPercent => "20%"
        }.fmt(f)
    }
}

pub fn map_pf_to_label(value: f64) -> String {
    fn print_short(x: f64) -> String {
        let y = format!("{:.1}", x);
        y.replace(".0", "")
    }
    if value < 1e3 {
        // pF case
        format!("{}pF", print_short(value))
    } else if value < 1e6 {
        // nF case
        format!("{}nF", print_short(value / 1e3))
    } else if value < 1e9 {
        // uF case
        format!("{}uF", print_short(value / 1e6))
    } else {
        // mF case??
        format!("{}mF", print_short(value / 1e9))
    }
}
