use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

use crate::bom::{Manufacturer, Supplier};
use crate::capacitors::{CapacitorKind, CapacitorTolerance};
use crate::designator::Designator;
use crate::diode::DiodeKind;
use crate::epin::EPin;
use crate::glyph::{Glyph, Point};
use crate::resistors::{PowerWatt, ResistorKind};
use crate::schematic_layout::{SchematicOrientation, SchematicRotation};
use crate::smd::SizeCode;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartDetails {
    pub label: String,
    pub manufacturer: Manufacturer,
    pub description: String,
    pub comment: String,
    pub hide_pin_designators: bool,
    pub hide_part_outline: bool,
    pub pins: BTreeMap<u64, EPin>,
    pub outline: Vec<Glyph>,
    pub size: SizeCode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capacitor {
    pub details: PartDetails,
    pub value_pf: f64,
    pub kind: CapacitorKind,
    pub voltage: f64,
    pub tolerance: CapacitorTolerance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resistor {
    pub details: PartDetails,
    pub value_ohms: f64,
    pub kind: ResistorKind,
    pub power_watt: PowerWatt,
    pub tolerance: f64,
    pub tempco: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inductor {
    pub details: PartDetails,
    pub value_microhenry: f64,
    pub tolerance: f64,
    pub dc_resistance_ohms: f64,
    pub max_current_milliamps: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diode {
    pub details: PartDetails,
    pub forward_drop_volts: f64,
    pub kind: DiodeKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Regulator {
    pub details: PartDetails,
    pub input_min_voltage: f64,
    pub input_max_voltage: f64,
    pub output_nominal_voltage: f64,
    pub output_max_current_ma: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LogicSignalStandard {
    CMOS3V3,
    TTL,
    WideRange,
    TriState,
    TriState5v0,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LogicFunction {
    XOR,
    Buffer,
    Decoder,
    Multiplexer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Logic {
    pub details: PartDetails,
    pub drive_current_ma: f64,
    pub min_supply_voltage: f64,
    pub max_supply_voltage: f64,
    pub input_type: LogicSignalStandard,
    pub output_type: LogicSignalStandard,
    pub function: LogicFunction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartPin {
    pub part_id: String,
    pub pin: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Net {
    pub pins: Vec<PartPin>,
    pub name: String,
}

impl Net {
    pub fn new(name: &str) -> Net {
        Net {
            pins: vec![],
            name: name.into(),
        }
    }
    pub fn add(mut self, from_part: &PartInstance, from_index: u64) -> Self {
        let from_pin = PartPin {
            part_id: from_part.id.clone(),
            pin: from_index,
        };
        self.pins.push(from_pin);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CircuitNode {
    Capacitor(Capacitor),
    Resistor(Resistor),
    Diode(Diode),
    Regulator(Regulator),
    Inductor(Inductor),
    IntegratedCircuit(PartDetails),
    Connector(PartDetails),
    Logic(Logic),
    Port(PartDetails),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Circuit {
    pub nodes: Vec<PartInstance>,
    pub nets: Vec<Net>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartInstance {
    pub node: CircuitNode,
    pub id: String,
}

pub fn instance(x: CircuitNode, name: &str) -> PartInstance {
    PartInstance {
        node: x,
        id: name.into(),
    }
}

impl CircuitNode {
    pub fn instance(self, name: &str) -> PartInstance {
        PartInstance {
            node: self,
            id: name.into(),
        }
    }
}
