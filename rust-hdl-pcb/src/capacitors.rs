use std::fmt::{Display, Formatter, Pointer};

pub enum CapacitanceValues {
    F100N,
    FU1,
    F1U,
    F4U7,
    F22U,
    F10U,
    F100U,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorkingVoltage {
    V2V5,
    V4,
    V6V3,
    V10,
    V16,
    V25,
    V35,
    V50,
    V75,
    V100,
    V250,
    V450,
    V630,
}

impl Display for WorkingVoltage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkingVoltage::V2V5 => "2.5V",
            WorkingVoltage::V4 => "4V",
            WorkingVoltage::V6V3 => "6.3V",
            WorkingVoltage::V10 => "10V",
            WorkingVoltage::V16 => "16V",
            WorkingVoltage::V25 => "25V",
            WorkingVoltage::V35 => "35V",
            WorkingVoltage::V50 => "50V",
            WorkingVoltage::V75 => "75V",
            WorkingVoltage::V100 => "100V",
            WorkingVoltage::V250 => "250V",
            WorkingVoltage::V450 => "450V",
            WorkingVoltage::V630 => "630V",
        }.fmt(f)
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
    MultiLayerChip,
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
