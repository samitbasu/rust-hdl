#[derive(Debug, Clone, PartialEq)]
pub enum SynthKind {
    Array { base: Box<SynthKind>, size: usize },
    Tuple { elements: Vec<SynthKind> },
    Struct { fields: Vec<(String, SynthKind)> },
    Enum { variants: Vec<(String, SynthKind)> },
    Bits { digits: Vec<bool> },
    Empty,
}

pub const fn clog2(t: usize) -> usize {
    let mut p = 0;
    let mut b = 1;
    while b < t {
        p += 1;
        b *= 2;
    }
    p
}

impl SynthKind {
    pub fn bits(&self) -> usize {
        match self {
            SynthKind::Array { base, size } => base.bits() * size,
            SynthKind::Tuple { elements } => elements.iter().map(|x| x.bits()).sum(),
            SynthKind::Struct { fields } => fields.iter().map(|x| x.1.bits()).sum(),
            SynthKind::Enum { variants } => {
                clog2(variants.len()) + variants.iter().map(|x| x.1.bits()).max().unwrap_or(0)
            }
            SynthKind::Bits { digits } => digits.len(),
            SynthKind::Empty => 0,
        }
    }
}
