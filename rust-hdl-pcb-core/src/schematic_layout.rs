use std::collections::BTreeMap;

use crate::glyph::Point;
use serde::{Deserialize, Serialize};

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
