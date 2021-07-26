use crate::bom::{Manufacturer, Supplier};
use crate::designator::Designator;
use crate::epin::{EPin, InputRange, OutputRange};
use crate::capacitors::{CapacitorKind, DielectricCode, CapacitorTolerance};
use crate::resistors::{PowerWatt, ResistorKind};
use crate::smd::SizeCode;

#[derive(Clone, Debug)]
pub struct PartDetails {
    pub label: String,
    pub manufacturer: Manufacturer,
    pub description: String,
    pub comment: String,
    pub pins: Vec<EPin>,
    pub suppliers: Vec<Supplier>,
    pub designator: Designator,
    pub size: SizeCode,
}

#[derive(Clone, Debug)]
pub struct Capacitor {
    pub details: PartDetails,
    pub value_pf: f64,
    pub kind: CapacitorKind,
    pub voltage: f64,
    pub tolerance: CapacitorTolerance,
}

#[derive(Clone, Debug)]
pub struct Resistor {
    pub details: PartDetails,
    pub value_ohms: f64,
    pub kind: ResistorKind,
    pub power_watt: PowerWatt,
    pub tolerance: f64,
    pub tempco: Option<f64>,
}

pub struct Net {
    source_pin: u64,
    dest_pin: u64,
    name: String,
}

pub enum CircuitNode {
    Capacitor(Capacitor),
    Resistor(Resistor),
    IntegratedCircuit(PartDetails),
    Circuit(Box<Circuit>),
}

pub struct Circuit {
    pins: Vec<EPin>,
    nodes: Vec<CircuitNode>,
    net: Vec<Net>,
}
