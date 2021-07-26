use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::string::ParseError;

#[derive(Clone, Debug, PartialEq)]
pub struct TolerancedDim {
    pub nominal_mm: f64,
    pub tolerance_mm: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PTHResistor {
    pub body_length: TolerancedDim,
    pub body_diameter: TolerancedDim,
    pub lead_length: TolerancedDim,
    pub lead_diameter: TolerancedDim,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SizeCode {
    I0075,
    I0100,
    I0201,
    I0204,
    I0402,
    I0603,
    I0805,
    I1206,
    I1210,
    I1218,
    I1812,
    I1825,
    I2010,
    I2220,
    I2512,
    I3025,
    PTHResistor(PTHResistor),
    Custom(String),
}

impl FromStr for SizeCode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "0075" => SizeCode::I0075,
            "0100" => SizeCode::I0100,
            "0201" => SizeCode::I0201,
            "0204" => SizeCode::I0204,
            "0402" => SizeCode::I0402,
            "0603" => SizeCode::I0603,
            "0805" => SizeCode::I0805,
            "1206" => SizeCode::I1206,
            "1210" => SizeCode::I1210,
            "1218" => SizeCode::I1218,
            "2010" => SizeCode::I2010,
            "2512" => SizeCode::I2512,
            "1812" => SizeCode::I1812,
            "1825" => SizeCode::I1825,
            "2220" => SizeCode::I2220,
            "3025" => SizeCode::I3025,
            _ => SizeCode::Custom(s.to_owned()),
        })
    }
}

impl Display for SizeCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SizeCode::I0075 => "0075",
            SizeCode::I0100 => "0100",
            SizeCode::I0201 => "0201",
            SizeCode::I0204 => "0204",
            SizeCode::I0402 => "0402",
            SizeCode::I0603 => "0603",
            SizeCode::I0805 => "0805",
            SizeCode::I1206 => "1206",
            SizeCode::I1210 => "1210",
            SizeCode::I1218 => "1218",
            SizeCode::I2010 => "2010",
            SizeCode::I2512 => "2512",
            SizeCode::I1812 => "1812",
            SizeCode::I1825 => "1825",
            SizeCode::I2220 => "2220",
            SizeCode::I3025 => "3025",
            SizeCode::PTHResistor(p) => "PTH",
            SizeCode::Custom(s) => s,
        }
        .fmt(f)
    }
}
