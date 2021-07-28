#[derive(Clone, Copy, Debug)]
pub struct InputRange {
    minimum: Option<f64>,
    maximum: Option<f64>,
    nominal: Option<f64>,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputRange {
    nominal_vdc: Option<f64>,
    max_current_ma: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PinKind {
    Input,
    InputInverted,
    Output,
    TriState,
    Passive,
    PassivePos,
    PassiveNeg,
    PowerSink,
    PowerSource,
    PowerReturn,
    OpenCollector,
    OpenEmitter,
    NoConnect,
    Free,
    Unspecified,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EdgeLocation {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PinLocation {
    pub offset: i32,
    pub edge: EdgeLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EPin {
    pub kind: PinKind,
    pub name: String,
    pub location: PinLocation,
}

impl EPin {
    pub fn new(name: &str, kind: PinKind, location: PinLocation) -> Self {
        Self {
            kind,
            name: name.into(),
            location,
        }
    }
    pub fn old(name: &str, kind: PinKind) -> Self {
        Self::new(name, kind, PinLocation {
            offset: 0,
            edge: EdgeLocation::North
        })
    }
    pub fn passive(location: PinLocation) -> Self {
        EPin {
            kind: PinKind::Passive,
            name: "".to_string(),
            location,
        }
    }
    pub fn passive_pos() -> Self {
        EPin {
            kind: PinKind::PassivePos,
            name: "".to_string(),
            location: PinLocation {
                offset: 0,
                edge: EdgeLocation::West
            },
        }
    }
    pub fn passive_neg() -> Self {
        EPin {
            kind: PinKind::PassiveNeg,
            name: "".to_string(),
            location: PinLocation {
                offset: 0,
                edge: EdgeLocation::East
            }
        }
    }
    pub fn is_named(&self, name: &str) -> bool {
        self.name.eq(name)
    }
    pub fn is_type(&self, kind: PinKind) -> bool {
        self.kind.eq(&kind)
    }
}

pub fn make_passive_pin_pair() -> Vec<EPin> {
    vec![
        EPin::passive(PinLocation {
            offset: 0,
            edge: EdgeLocation::West,
        }),
        EPin::passive(PinLocation {
            offset: 0,
            edge: EdgeLocation::East,
        })
    ]
}

#[macro_export]
macro_rules! pin {
    ($name:expr, $kind:ident, $pos: expr, $edge: ident) => {
        EPin::new($name, crate::epin::PinKind::$kind, crate::epin::PinLocation {
            offset: $pos,
            edge: crate::epin::EdgeLocation::$edge
        })
    }
}
