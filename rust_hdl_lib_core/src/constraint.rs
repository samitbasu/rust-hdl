#![allow(non_camel_case_types)]

#[derive(Clone, Debug)]
pub enum SignalType {
    LowVoltageCMOS_1v8,
    LowVoltageCMOS_3v3,
    StubSeriesTerminatedLogic_II,
    DifferentialStubSeriesTerminatedLogic_II,
    StubSeriesTerminatedLogic_II_No_Termination,
    DifferentialStubSeriesTerminatedLogic_II_No_Termination,
    LowVoltageDifferentialSignal_2v5,
    StubSeriesTerminatedLogic_1v5,
    LowVoltageCMOS_1v5,
    DifferentialStubSeriesTerminatedLogic_1v5,
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct PeriodicTiming {
    pub net: String,
    pub period_nanoseconds: f64,
    pub duty_cycle: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum TimingRelative {
    Before,
    After,
}

impl ToString for TimingRelative {
    fn to_string(&self) -> String {
        match self {
            TimingRelative::Before => "BEFORE",
            TimingRelative::After => "AFTER",
        }
        .into()
    }
}

#[derive(Clone, Debug)]
pub struct FalsePathRegexp {
    pub from_regexp: String,
    pub to_regexp: String,
}

#[derive(Clone, Copy, Debug)]
pub enum TimingRelativeEdge {
    Rising,
    Falling,
}

impl ToString for TimingRelativeEdge {
    fn to_string(&self) -> String {
        match self {
            TimingRelativeEdge::Falling => "FALLING",
            TimingRelativeEdge::Rising => "RISING",
        }
        .into()
    }
}

#[derive(Clone, Debug)]
pub struct VivadoInputTimingConstraint {
    pub min_nanoseconds: f64,
    pub max_nanoseconds: f64,
    pub multicycle: usize,
    pub clock: String,
}

#[derive(Clone, Debug)]
pub struct VivadoOutputTimingConstraint {
    pub delay_nanoseconds: f64,
    pub clock: String,
}

#[derive(Clone, Copy, Debug)]
pub struct InputTimingConstraint {
    pub offset_nanoseconds: f64,
    pub valid_duration_nanoseconds: f64,
    pub relative: TimingRelative,
    pub edge_sense: TimingRelativeEdge,
    pub to_signal_id: usize,
    pub to_signal_bit: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputTimingConstraint {
    pub offset_nanoseconds: f64,
    pub relative: TimingRelative,
    pub edge_sense: TimingRelativeEdge,
    pub to_signal_id: usize,
    pub to_signal_bit: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum Timing {
    Periodic(PeriodicTiming),
    InputTiming(InputTimingConstraint),
    OutputTiming(OutputTimingConstraint),
    VivadoInputTiming(VivadoInputTimingConstraint),
    VivadoOutputTiming(VivadoOutputTimingConstraint),
    VivadoClockGroup(Vec<Vec<String>>),
    VivadoFalsePath(FalsePathRegexp),
    Custom(String),
}

#[derive(Clone, Debug)]
pub enum Constraint {
    Location(String),
    Kind(SignalType),
    Timing(Timing),
    Custom(String),
    Slew(SlewType),
}

#[derive(Clone, Debug)]
pub enum SlewType {
    Normal,
    Fast,
}

#[derive(Clone, Debug)]
pub struct PinConstraint {
    pub index: usize,
    pub constraint: Constraint,
}
