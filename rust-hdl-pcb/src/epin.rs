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
pub struct EPin {
    pub kind: PinKind,
    pub name: String,
}

impl EPin {
    pub fn new(name: &str, kind: PinKind) -> Self {
        Self {
            kind,
            name: name.into()
        }
    }
    pub fn passive() -> Self {
        EPin {
            kind: PinKind::Passive,
            name: "".to_string(),
        }
    }
    pub fn passive_pos() -> Self {
        EPin {
            kind: PinKind::PassivePos,
            name: "".to_string(),
        }
    }
    pub fn passive_neg() -> Self {
        EPin {
            kind: PinKind::PassiveNeg,
            name: "".to_string(),
        }
    }


}
