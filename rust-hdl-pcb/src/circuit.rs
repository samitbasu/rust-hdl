use std::collections::BTreeMap;

use crate::bom::{Manufacturer, Supplier};
use crate::capacitors::{CapacitorKind, CapacitorTolerance};
use crate::designator::Designator;
use crate::diode::DiodeKind;
use crate::epin::EPin;
use crate::glyph::{Glyph, Point};
use crate::resistors::{PowerWatt, ResistorKind};
use crate::smd::SizeCode;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SchematicRotation {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug)]
pub struct SchematicOrientation {
    pub rotation: SchematicRotation,
    pub flipped_lr: bool,
    pub flipped_ud: bool,
    pub center: Point,
}

impl Default for SchematicOrientation {
    fn default() -> Self {
        Self {
            rotation: SchematicRotation::Horizontal,
            flipped_lr: false,
            flipped_ud: false,
            center: Point::zero(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PartDetails {
    pub label: String,
    pub manufacturer: Manufacturer,
    pub description: String,
    pub comment: String,
    pub hide_pin_designators: bool,
    pub pins: BTreeMap<u64, EPin>,
    pub outline: Vec<Glyph>,
    pub suppliers: Vec<Supplier>,
    pub designator: Designator,
    pub size: SizeCode,
}

static GLOBAL_PART_COUNT: AtomicUsize = AtomicUsize::new(1);

pub fn get_part_id() -> PartID {
    PartID(GLOBAL_PART_COUNT.fetch_add(1, Ordering::SeqCst))
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct PartID(pub(crate) usize);

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

#[derive(Clone, Debug)]
pub struct Inductor {
    pub details: PartDetails,
    pub value_microhenry: f64,
    pub tolerance: f64,
    pub dc_resistance_ohms: f64,
    pub max_current_milliamps: f64,
}

#[derive(Clone, Debug)]
pub struct Diode {
    pub details: PartDetails,
    pub forward_drop_volts: f64,
    pub kind: DiodeKind,
}

#[derive(Clone, Debug)]
pub struct Regulator {
    pub details: PartDetails,
    pub input_min_voltage: f64,
    pub input_max_voltage: f64,
    pub output_nominal_voltage: f64,
    pub output_max_current_ma: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogicSignalStandard {
    CMOS3V3,
    TTL,
    WideRange,
    TriState,
    TriState5v0,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogicFunction {
    XOR,
    Buffer,
    Decoder,
    Multiplexer,
}

#[derive(Clone, Debug)]
pub struct Logic {
    pub details: PartDetails,
    pub drive_current_ma: f64,
    pub min_supply_voltage: f64,
    pub max_supply_voltage: f64,
    pub input_type: LogicSignalStandard,
    pub output_type: LogicSignalStandard,
    pub function: LogicFunction,
}

#[derive(Clone, Debug, Copy)]
pub struct PartPin {
    pub part_id: PartID,
    pub pin: u64,
}

#[derive(Clone, Debug)]
pub enum LogicalTracePoint {
    Part(PartPin),
    Node(usize),
}

#[derive(Clone, Debug)]
pub struct LogicalTrace {
    pub start: LogicalTracePoint,
    pub waypoints: Vec<(i32, i32)>,
    pub end: LogicalTracePoint,
}

#[derive(Clone, Debug)]
pub struct Net {
    pub pins: Vec<PartPin>,
    pub logical_nodes: Vec<(i32, i32)>,
    pub logical: Vec<LogicalTrace>,
    pub name: Option<String>,
}

impl Net {
    pub fn new(name: Option<&str>) -> Net {
        Net {
            pins: Default::default(),
            logical_nodes: vec![],
            logical: vec![],
            name: name.map(|x| x.into()),
        }
    }
    pub fn add(mut self, part: &PartInstance, index: u64) -> Self {
        let pin = PartPin {
            part_id: part.id.clone(),
            pin: index,
        };
        self.pins.push(pin);
        self
    }
    pub fn add_port(mut self, index: u64) -> Self {
        let pin = PartPin {
            part_id: PartID(0),
            pin: index,
        };
        self.pins.push(pin);
        self
    }
    pub fn add_logical_node(mut self, pos: (i32, i32)) -> Self {
        self.logical_nodes.push(pos);
        self
    }
}

#[derive(Clone, Debug)]
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

#[derive(Debug)]
pub struct Circuit {
    pub nodes: Vec<PartInstance>,
    pub nets: Vec<Net>,
}

#[derive(Debug)]
pub struct PartInstance {
    pub node: CircuitNode,
    pub schematic_orientation: SchematicOrientation,
    pub id: PartID,
}

impl PartInstance {
    pub fn rot90(mut self) -> Self {
        self.schematic_orientation.rotation = SchematicRotation::Vertical;
        self
    }
    pub fn flip_lr(mut self) -> Self {
        self.schematic_orientation.flipped_lr = !self.schematic_orientation.flipped_lr;
        self
    }
    pub fn flip_ud(mut self) -> Self {
        self.schematic_orientation.flipped_ud = !self.schematic_orientation.flipped_ud;
        self
    }
}

impl From<CircuitNode> for PartInstance {
    fn from(x: CircuitNode) -> Self {
        PartInstance {
            node: x,
            schematic_orientation: SchematicOrientation::default(),
            id: get_part_id(),
        }
    }
}
