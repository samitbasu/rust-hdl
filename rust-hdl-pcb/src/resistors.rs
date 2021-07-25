use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResistorTolerance {
    TenthPercent,
    HalfPercent,
    OnePercent,
    FivePercent,
}

impl Display for ResistorTolerance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResistorTolerance::TenthPercent => "0.1%",
            ResistorTolerance::HalfPercent => "0.5%",
            ResistorTolerance::OnePercent => "1%",
            ResistorTolerance::FivePercent => "5%",
        }
        .fmt(f)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PowerMilliWatt {
    MW20,
    MW31P25,
    MW50,
    MW62P5,
    MW100,
    MW125,
    MW250,
    MW500,
    MW750,
    MW1000,
    MW2000,
}

impl Display for PowerMilliWatt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerMilliWatt::MW20 => "1/50W",
            PowerMilliWatt::MW31P25 => "1/32W",
            PowerMilliWatt::MW50 => "1/20W",
            PowerMilliWatt::MW62P5 => "1/16W",
            PowerMilliWatt::MW100 => "1/10W",
            PowerMilliWatt::MW125 => "1/8W",
            PowerMilliWatt::MW250 => "1/4W",
            PowerMilliWatt::MW500 => "1/2W",
            PowerMilliWatt::MW750 => "3/4W",
            PowerMilliWatt::MW1000 => "1W",
            PowerMilliWatt::MW2000 => "2W",
        }
        .fmt(f)
    }
}

impl PowerMilliWatt {
    pub fn as_value(&self) -> f64 {
        match self {
            PowerMilliWatt::MW20 => 20.0,
            PowerMilliWatt::MW31P25 => 31.25,
            PowerMilliWatt::MW50 => 50.0,
            PowerMilliWatt::MW62P5 => 62.5,
            PowerMilliWatt::MW100 => 100.0,
            PowerMilliWatt::MW125 => 125.0,
            PowerMilliWatt::MW250 => 250.0,
            PowerMilliWatt::MW500 => 500.0,
            PowerMilliWatt::MW750 => 750.0,
            PowerMilliWatt::MW1000 => 1000.0,
            PowerMilliWatt::MW2000 => 2000.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ResistorKind {
    ThinFilmChip,
    ThickFilmChip,
    WireWound,
    Carbon
}
