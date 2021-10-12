use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::circuit::{Circuit, PartPin};
use crate::epin::{EPin, EdgeLocation};
use crate::glyph::{Glyph, Point, Rect};
use crate::prelude::get_details_from_instance;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SchematicRotation {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SchematicOrientation {
    pub rotation: SchematicRotation,
    pub flipped_lr: bool,
    pub flipped_ud: bool,
    pub center: (i32, i32),
}

pub fn orient() -> SchematicOrientation {
    SchematicOrientation::default()
}

impl Default for SchematicOrientation {
    fn default() -> Self {
        Self {
            rotation: SchematicRotation::Horizontal,
            flipped_lr: false,
            flipped_ud: false,
            center: (0, 0),
        }
    }
}

impl SchematicOrientation {
    pub fn flip_lr(self) -> Self {
        Self {
            flipped_lr: !self.flipped_lr,
            ..self
        }
    }

    pub fn flip_ud(self) -> Self {
        Self {
            flipped_ud: !self.flipped_ud,
            ..self
        }
    }

    pub fn vert(self) -> Self {
        Self {
            rotation: SchematicRotation::Vertical,
            ..self
        }
    }

    pub fn horiz(self) -> Self {
        Self {
            rotation: SchematicRotation::Horizontal,
            ..self
        }
    }

    pub fn center(self, px: i32, py: i32) -> Self {
        Self {
            center: (px, py),
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetLayoutCmd {
    MoveToPort(usize),
    LineToPort(usize),
    MoveToCoords(i32, i32),
    LineToCoords(i32, i32),
    Junction,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchematicLayout {
    pub parts: BTreeMap<String, SchematicOrientation>,
    pub nets: BTreeMap<String, Vec<NetLayoutCmd>>,
}

pub fn make_rat_layout(num_ports: usize) -> Vec<NetLayoutCmd> {
    let mut ret = vec![];
    ret.push(NetLayoutCmd::MoveToPort(1));
    for i in 1..num_ports {
        ret.push(NetLayoutCmd::LineToPort(i + 1));
    }
    ret
}

impl SchematicLayout {
    pub fn part(&self, part: &str) -> SchematicOrientation {
        match self.parts.get(part) {
            None => SchematicOrientation::default(),
            Some(o) => o.clone(),
        }
    }
    pub fn set_part(&mut self, part: &str, orient: SchematicOrientation) {
        self.parts.insert(part.into(), orient);
    }
    pub fn net(&self, net: &str) -> Vec<NetLayoutCmd> {
        match self.nets.get(net) {
            None => vec![],
            Some(v) => v.clone(),
        }
    }
    pub fn set_net(&mut self, net: &str, layout: Vec<NetLayoutCmd>) {
        self.nets.insert(net.into(), layout);
    }
}

pub const PIN_LENGTH: i32 = 200;

pub fn map_pin_based_on_orientation(orient: &SchematicOrientation, x: i32, y: i32) -> (i32, i32) {
    let cx = orient.center.0;
    let cy = orient.center.1;
    return match orient.rotation {
        SchematicRotation::Horizontal => (x + cx, -(y + cy)),
        SchematicRotation::Vertical => (-y + cx, -(x + cy)),
    };
}

pub fn map_pin_based_on_outline_and_orientation(
    pin: &EPin,
    r: &Rect,
    orientation: &SchematicOrientation,
    len: i32,
) -> (i32, i32) {
    return match &pin.location.edge {
        EdgeLocation::North => {
            map_pin_based_on_orientation(&orientation, pin.location.offset, r.p1.y + len)
        }
        EdgeLocation::West => {
            map_pin_based_on_orientation(&orientation, r.p0.x - len, pin.location.offset)
        }
        EdgeLocation::East => {
            map_pin_based_on_orientation(&orientation, r.p1.x + len, pin.location.offset)
        }
        EdgeLocation::South => {
            map_pin_based_on_orientation(&orientation, pin.location.offset, r.p0.y - len)
        }
    };
}

pub fn get_pin_net_location(
    circuit: &Circuit,
    layout: &SchematicLayout,
    pin: &PartPin,
) -> (i32, i32) {
    for instance in &circuit.nodes {
        if instance.id == pin.part_id {
            let part = get_details_from_instance(instance, layout);
            let schematic_orientation = layout.part(&instance.id);
            let pin = &part.pins[&pin.pin];
            return if let Glyph::OutlineRect(r) = &part.outline[0] {
                map_pin_based_on_outline_and_orientation(pin, r, &schematic_orientation, PIN_LENGTH)
            } else {
                // Parts without an outline rect are just virtual...
                (
                    schematic_orientation.center.0,
                    -schematic_orientation.center.1,
                )
            };
        }
    }
    panic!("No pin found!")
}
