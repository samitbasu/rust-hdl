use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResistorTempco {
    Ppm15degC,
    Ppm25degC,
    Ppm50degC,
}

impl Display for ResistorTempco {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResistorTempco::Ppm15degC => "15 ppm/C",
            ResistorTempco::Ppm25degC => "25 ppm/C",
            ResistorTempco::Ppm50degC => "50 ppm/C",
        }.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResistorTolerance {
    TenthPercent,
    HalfPercent,
    OnePercent,
    TwoPercent,
    FivePercent,
}

impl Display for ResistorTolerance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResistorTolerance::TenthPercent => "0.1%",
            ResistorTolerance::HalfPercent => "0.5%",
            ResistorTolerance::OnePercent => "1%",
            ResistorTolerance::TwoPercent => "2%",
            ResistorTolerance::FivePercent => "5%",
        }
        .fmt(f)
    }
}

pub type PowerWatt = num_rational::Rational32;

#[derive(Clone, Copy, Debug)]
pub enum ResistorKind {
    ThinFilmChip,
    ThickFilmChip,
    WireWound,
    Carbon
}
