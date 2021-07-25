use crate::bom::{Manufacturer, Supplier};
use crate::designator::Designator;
use crate::epin::{EPin, InputRange, OutputRange};

#[derive(Clone, Debug)]
pub struct Part {
    pub label: String,
    pub manufacturer: Manufacturer,
    pub description: String,
    pub comment: String,
    pub pins: Vec<EPin>,
    pub suppliers: Vec<Supplier>,
    pub datasheet: Option<url::Url>,
    pub designator: Designator,
}

pub struct Net {
    source_pin: u64,
    dest_pin: u64,
    name: String,
}

pub enum CircuitNode {
    Part(Part),
    Circuit(Circuit),
}

pub struct Circuit {
    pins: Vec<EPin>,
    nodes: Vec<CircuitNode>,
    net: Vec<Net>,
}
