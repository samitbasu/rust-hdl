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
pub enum SignalKind {
    Any,
    Logic5V,
    Logic3V3,
    Clock,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PassiveKind {
    Any,
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PinKind {
    Input(SignalKind),
    Output(SignalKind),
    Bidirectional(SignalKind),
    TriState(SignalKind),
    Passive(PassiveKind),
    PowerSink,
    PowerSource,
    PowerReturn,
    OpenCollector,
    OpenEmitter,
    NoConnect,
    Free,
    Unspecified,
}

#[derive(Clone, Debug)]
pub struct EPin {
    pub kind: PinKind,
    pub name: String,
    pub designator_visible: bool,
}

impl EPin {
    pub fn passive() -> Self {
        EPin {
            kind: PinKind::Passive(PassiveKind::Any),
            name: "".to_string(),
            designator_visible: false,
        }
    }
    pub fn passive_pos() -> Self {
        EPin {
            kind: PinKind::Passive(PassiveKind::Positive),
            name: "".to_string(),
            designator_visible: false,
        }
    }
    pub fn passive_neg() -> Self {
        EPin {
            kind: PinKind::Passive(PassiveKind::Negative),
            name: "".to_string(),
            designator_visible: false,
        }
    }


}
