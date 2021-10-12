use std::collections::BTreeMap;

use crate::circuit::{CircuitNode, PartDetails, PartInstance};
use crate::epin::EPin;
use crate::prelude::SchematicLayout;

pub fn drop_char(txt: &str) -> &str {
    let len = txt.len();
    &txt[..(len - 1)]
}

pub fn pin_list(pins: Vec<EPin>) -> BTreeMap<u64, EPin> {
    let mut map = BTreeMap::new();
    for pin in pins.into_iter().enumerate() {
        map.insert((pin.0 + 1) as u64, pin.1);
    }
    map
}

pub fn make_flip_lr_part(part: &PartDetails) -> PartDetails {
    let mut fpart = part.clone();
    fpart.outline = part.outline.iter().map(|x| x.fliplr()).collect();
    fpart.pins = part
        .pins
        .iter()
        .map(|x| (*x.0, EPin::new(&x.1.name, x.1.kind, x.1.location.fliplr())))
        .collect();
    fpart
}

pub fn make_flip_ud_part(part: &PartDetails) -> PartDetails {
    let mut fpart = part.clone();
    fpart.outline = part.outline.iter().map(|x| x.flipud()).collect();
    fpart.pins = part
        .pins
        .iter()
        .map(|x| (*x.0, EPin::new(&x.1.name, x.1.kind, x.1.location.flipud())))
        .collect();
    fpart
}

pub fn get_details_from_instance(x: &PartInstance, l: &SchematicLayout) -> PartDetails {
    let mut part = match &x.node {
        CircuitNode::Capacitor(c) => &c.details,
        CircuitNode::Resistor(r) => &r.details,
        CircuitNode::Diode(d) => &d.details,
        CircuitNode::Regulator(v) => &v.details,
        CircuitNode::Inductor(l) => &l.details,
        CircuitNode::IntegratedCircuit(u) => u,
        CircuitNode::Connector(j) => j,
        CircuitNode::Logic(u) => &u.details,
        CircuitNode::Port(p) => p,
    }
    .clone();

    let layout = l.part(&x.id);
    if layout.flipped_lr {
        part = make_flip_lr_part(&part);
    }

    if layout.flipped_ud {
        part = make_flip_ud_part(&part);
    }

    part
}
