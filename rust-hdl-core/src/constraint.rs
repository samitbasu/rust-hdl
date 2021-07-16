#![allow(non_camel_case_types)]

#[derive(Clone, Debug)]
pub enum SignalType {
    LowVoltageCMOS_1v8,
    LowVoltageCMOS_3v3,
    StubSeriesTerminatedLogic_II,
    DifferentialStubSeriesTerminatedLogic_II,
    StubSeriesTerminatedLogic_II_No_Termination,
    DifferentialStubSeriesTerminatedLogic_II_No_Termination,
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct PeriodicTiming {
    pub net: String,
    pub period_nanoseconds: f64,
    pub duty_cycle: f64,
}

#[derive(Clone, Debug)]
pub enum Timing {
    Periodic(PeriodicTiming),
    Custom(String),
}

#[derive(Clone, Debug)]
pub enum Constraint {
    Location(String),
    Kind(SignalType),
    Timing(Timing),
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct PinConstraint {
    pub index: usize,
    pub constraint: Constraint,
}
