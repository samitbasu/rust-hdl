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

#[derive(Clone, Copy, Debug)]
pub enum SignalKind {
    Any,
    Logic5V,
    Logic3V3,
    Clock,
    Custom,
}

#[derive(Clone, Copy, Debug)]
pub enum PinKind {
    Input(SignalKind),
    Output(SignalKind),
    Bidirectional(SignalKind),
    TriState(SignalKind),
    Passive,
    PowerSink(InputRange),
    PowerSource(OutputRange),
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
    pub designator: u64,
    pub name: String,
    pub designator_visible: bool,
}

impl EPin {
    pub fn passive(designator: u64) -> Self {
        EPin {
            kind: PinKind::Passive,
            designator,
            name: "".to_string(),
            designator_visible: false,
        }
    }
}
