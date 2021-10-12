use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::bom::Manufacturer;
use crate::circuit::{Capacitor, CircuitNode, PartDetails};
use crate::designator::{Designator, DesignatorKind};
use crate::epin::make_passive_pin_pair;
use crate::glyph::{make_arc, make_ic_body, make_label, make_line, Glyph, TextJustification};
use crate::smd::SizeCode;
use crate::utils::pin_list;

pub fn map_three_digit_cap_to_uf(uf: &str) -> f64 {
    let uf_tens = &uf[0..1].parse::<f64>().unwrap();
    let uf_ones = &uf[1..2].parse::<f64>().unwrap();
    let uf_exp = &uf[2..3].parse::<f64>().unwrap();
    (uf_tens * 10.0 + uf_ones) * 10.0_f64.powf(*uf_exp)
}

pub fn map_three_digit_cap_to_pf(pf: &str) -> f64 {
    return if &pf[1..2] == "R" {
        let pf_ones = &pf[0..1].parse::<f64>().unwrap();
        let pf_tenths = &pf[2..3].parse::<f64>().unwrap();
        pf_ones + pf_tenths * 0.1
    } else if &pf[0..1] == "R" {
        let pf_tenths = &pf[1..2].parse::<f64>().unwrap();
        let pf_hundredths = &pf[2..3].parse::<f64>().unwrap();
        pf_tenths * 0.1 + pf_hundredths * 0.001
    } else {
        let pf_tens = &pf[0..1].parse::<f64>().unwrap();
        let pf_ones = &pf[1..2].parse::<f64>().unwrap();
        let pf_exp = &pf[2..3].parse::<f64>().unwrap();
        (pf_tens * 10.0 + pf_ones) * 10.0_f64.powf(*pf_exp)
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DielectricCode {
    X5R,
    X7R,
    C0G,
    X7T,
}

impl FromStr for DielectricCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "X5R" => DielectricCode::X5R,
            "X7R" => DielectricCode::X7R,
            "C0G" => DielectricCode::C0G,
            "NP0" => DielectricCode::C0G,
            "X7T" => DielectricCode::X7T,
            _ => panic!("Unsupported dielectric code {}", s),
        })
    }
}

impl Display for DielectricCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DielectricCode::X5R => "X5R",
            DielectricCode::X7R => "X7R",
            DielectricCode::C0G => "C0G",
            DielectricCode::X7T => "X7T",
        }
        .fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CapacitorKind {
    MultiLayerChip(DielectricCode),
    Tantalum,
    AluminumPolyLowESR(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CapacitorTolerance {
    TenthPF,
    QuarterPF,
    HalfPF,
    OnePercent,
    TwoPercent,
    FivePercent,
    TenPercent,
    TwentyPercent,
}

impl Display for CapacitorTolerance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CapacitorTolerance::TenthPF => "0.1PF",
            CapacitorTolerance::QuarterPF => "0.25PF",
            CapacitorTolerance::HalfPF => "0.5PF",
            CapacitorTolerance::OnePercent => "1%",
            CapacitorTolerance::TwoPercent => "2%",
            CapacitorTolerance::FivePercent => "5%",
            CapacitorTolerance::TenPercent => "10%",
            CapacitorTolerance::TwentyPercent => "20%",
        }
        .fmt(f)
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

/*
   (arc (pt 170 0) (radius 99.963) (startAngle 135.852) (sweepAngle 89.47) (width 10))
   (line (pt -10 60) (pt -10 20) (width 10) )
   (line (pt -30 40) (pt 10 40) (width 10) )

*/
pub fn make_polarized_capacitor_outline(label: &str) -> Vec<Glyph> {
    let line1: String = label.split(" ").take(2).collect::<Vec<_>>().join(" ");
    let line2: String = label.split(" ").skip(2).collect::<Vec<_>>().join(" ");
    vec![
        make_ic_body(-30, -70, 100, 70),
        make_line(30, 70, 30, -70),
        make_line(30, 0, -30, 0),
        make_line(70, 0, 100, 0),
        make_line(-10, 60, -10, 20),
        make_line(-30, 40, 10, 40),
        make_arc(170, 0, 100.0, 135.0, 90.0),
        make_label(-10, 80, "C?", TextJustification::BottomLeft),
        make_label(-200, -80, &line1, TextJustification::TopLeft),
        make_label(-200, -180, &line2, TextJustification::TopLeft),
    ]
}

pub fn make_unpolarized_capacitor(
    label: String,
    manufacturer: Manufacturer,
    description: String,
    size: SizeCode,
) -> PartDetails {
    let line1: String = label.split(" ").take(2).collect::<Vec<_>>().join(" ");
    let line2: String = label.split(" ").skip(2).collect::<Vec<_>>().join(" ");
    PartDetails {
        label: label.clone(),
        manufacturer,
        description,
        comment: "".to_string(),
        hide_pin_designators: true,
        hide_part_outline: true,
        pins: pin_list(make_passive_pin_pair()),
        outline: vec![
            make_ic_body(0, -70, 100, 70),
            make_line(30, 70, 30, -70),
            make_line(70, 70, 70, -70),
            make_line(30, 0, 0, 0),
            make_line(70, 0, 100, 0),
            make_label(-10, 80, "C?", TextJustification::BottomLeft),
            make_label(-200, -80, &line1, TextJustification::TopLeft),
            make_label(-200, -180, &line2, TextJustification::TopLeft),
        ],
        size,
    }
}

pub fn make_mlcc(
    label: String,
    manufacturer: Manufacturer,
    description: String,
    size: SizeCode,
    value_pf: f64,
    dielectric: DielectricCode,
    voltage: f64,
    tolerance: CapacitorTolerance,
) -> CircuitNode {
    CircuitNode::Capacitor(Capacitor {
        details: make_unpolarized_capacitor(label, manufacturer, description, size),
        value_pf,
        kind: CapacitorKind::MultiLayerChip(dielectric),
        voltage,
        tolerance,
    })
}
