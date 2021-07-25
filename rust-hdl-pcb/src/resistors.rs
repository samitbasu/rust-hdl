use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub enum Tolerance {
    TenthPercent,
    HalfPercent,
    OnePercent,
    FivePercent,
}

impl Display for Tolerance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tolerance::TenthPercent => "0.1%",
            Tolerance::HalfPercent => "0.5%",
            Tolerance::OnePercent => "1%",
            Tolerance::FivePercent => "5%",
        }
        .fmt(f)
    }
}

pub enum PowerWatt {
    Fiftieth,
    ThirtySecond,
    Twentieth,
    Sixteenth,
    Tenth,
    Eighth,
    Quarter,
    Half,
    ThreeQuarter,
    One,
    Two,
}

impl Display for PowerWatt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerWatt::Fiftieth => "1/50W",
            PowerWatt::ThirtySecond => "1/32W",
            PowerWatt::Twentieth => "1/20W",
            PowerWatt::Sixteenth => "1/16W",
            PowerWatt::Tenth => "1/10W",
            PowerWatt::Eighth => "1/8W",
            PowerWatt::Quarter => "1/4W",
            PowerWatt::Half => "1/2W",
            PowerWatt::ThreeQuarter => "3/4W",
            PowerWatt::One => "1W",
            PowerWatt::Two => "2W",
        }
        .fmt(f)
    }
}

impl PowerWatt {
    pub fn as_value(&self) -> f64 {
        match self {
            PowerWatt::Fiftieth => 1. / 50.0,
            PowerWatt::ThirtySecond => 1. / 32.,
            PowerWatt::Twentieth => 1. / 20.,
            PowerWatt::Sixteenth => 1. / 16.,
            PowerWatt::Tenth => 1. / 10.,
            PowerWatt::Eighth => 1. / 8.,
            PowerWatt::Quarter => 1. / 4.,
            PowerWatt::Half => 1. / 2.,
            PowerWatt::ThreeQuarter => 3. / 4.,
            PowerWatt::One => 1.0,
            PowerWatt::Two => 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ResistanceValues {
    Ohm8R06,
    Ohm10R,
    Ohm16R,
    Ohm22R,
    Ohm47R,
    Ohm61R9,
    Ohm200R,
    Ohm240R,
    Ohm499R,
    Ohm910R,
    Ohm1K,
    Ohm5K6,
    Ohm10K,
    Ohm15K,
    Ohm20K5,
    Ohm39K,
    Ohm68K,
    Ohm100K,
}
