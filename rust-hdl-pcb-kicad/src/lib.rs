#![allow(non_camel_case_types)]

use rust_hdl_pcb_core::prelude::*;
use std::fmt::Display;
use std::str::FromStr;

use serde::{ser, Serialize};
use uuid::Uuid;

use crate::glyph::{polyline, rectangle};
use crate::pins::pin;
use crate::serde_s::to_s_string;
use rust_hdl_pcb_core::epin;
use std::collections::BTreeMap;

pub mod serde_s;
mod test;

#[derive(Debug, Clone, Serialize)]
pub enum FontDetail {
    size(f64, f64),
}

#[derive(Debug, Clone, Serialize)]
pub enum Justification {
    right,
    left,
    bottom,
    top,
}

#[derive(Debug, Clone, Serialize)]
pub enum Generator {
    eeschema,
}

#[derive(Debug, Clone, Serialize)]
pub enum Effect {
    font(Vec<FontDetail>),
    justify(Vec<Justification>),
    hide,
}

#[derive(Debug, Clone, Serialize)]
pub enum StrokeKind {
    solid,
}

#[derive(Debug, Clone, Serialize)]
pub enum StrokeDetails {
    width(f64),
    #[serde(rename = "type")]
    kind(StrokeKind),
    color(f64, f64, f64, f64),
}

#[derive(Debug, Clone, Serialize)]
pub enum Visual {
    fill(Fill),
    stroke(Vec<StrokeDetails>),
}

pub fn make_stroke_width(width: f64) -> Vec<StrokeDetails> {
    vec![StrokeDetails::width(width)]
}

#[derive(Debug, Clone, Serialize)]
pub enum FillType {
    none,
    background,
}

#[derive(Debug, Clone, Serialize)]
pub struct xy(f64, f64);

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "type")]
pub struct Fill(FillType);

#[derive(Debug, Clone, Serialize)]
pub enum KPinKind {
    power_in,
    input,
    power_out,
    no_connect,
    output,
    bidirectional,
    tri_state,
    passive,
    unspecified,
    open_collector,
    open_emitter,
    free,
}

#[derive(Debug, Clone, Serialize)]
pub enum PinAppearance {
    line,
}

#[derive(Debug, Clone, Serialize)]
pub enum PinHide {
    hide,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "pin_numbers")]
pub struct PinNumberOption {
    _hide: PinHide,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "fields_autoplaced")]
pub struct AutoFields {}

fn make_font_size(sze: f64) -> Effects {
    Effects(vec![Effect::font(vec![FontDetail::size(sze, sze)])])
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "radius")]
pub struct ArcDetails {
    at: (f64, f64),
    length: f64,
    angles: (f64, f64),
}

#[derive(Debug, Clone, Serialize)]
pub enum glyph {
    rectangle {
        start: (f64, f64),
        end: (f64, f64),
        _visuals: Vec<Visual>,
    },
    polyline {
        pts: Vec<xy>,
        _visuals: Vec<Visual>,
    },
    text {
        _text: String,
        at: (f64, f64, f64),
        _effects: Effects,
    },
    circle {
        center: (f64, f64),
        radius: f64,
        _visuals: Vec<Visual>,
    },
    arc {
        start: (f64, f64),
        end: (f64, f64),
        _details: ArcDetails,
        _visuals: Vec<Visual>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "effects")]
pub struct Effects(Vec<Effect>);

#[derive(Debug, Clone, Serialize)]
pub enum pins {
    pin {
        _kind: KPinKind,
        _appears: PinAppearance,
        at: (f64, f64, f64),
        length: f64,
        _hide: Option<PinHide>,
        name: (String, Effects),
        number: (String, Effects),
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "symbol")]
pub struct shape {
    _name: String,
    _elements: Vec<glyph>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "symbol")]
pub struct pinout {
    _name: String,
    _elements: Vec<pins>,
}

#[derive(Debug, Clone, Serialize)]
pub struct property {
    _name: String,
    _value: String,
    id: u32,
    at: (f64, f64, f64),
    effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "symbol")]
pub struct LibrarySymbol {
    _name: String,
    _pin_numbers: Option<PinNumberOption>,
    in_bom: bool,
    on_board: bool,
    _properties: Vec<property>,
    _shape: shape,
    _pinout: pinout,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "path")]
pub struct Page {
    _path: String,
    page: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum Element {
    lib_symbols(Vec<LibrarySymbol>),
    junction {
        at: (f64, f64),
        diameter: f64,
        color: (f64, f64, f64, f64),
    },
    no_connect {
        at: (f64, f64, f64),
        uuid: Uuid,
    },
    wire {
        pts: Vec<xy>,
        stroke: Vec<StrokeDetails>,
        uuid: Uuid,
    },
    label {
        _name: String,
        at: (f64, f64, f64),
        justify: Vec<Justification>,
        uuid: Uuid,
    },
    symbol {
        lib_id: String,
        at: (f64, f64, f64),
        unit: i32,
        in_bom: bool,
        on_board: bool,
        _auto: Option<AutoFields>,
        uuid: Uuid,
        _properties: Vec<property>,
        _pin: Vec<PinMap>,
    },
    sheet_instances {
        _pages: Vec<Page>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "pin")]
pub struct PinMap {
    _number: String,
    uuid: Uuid,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "kicad_sch")]
pub struct KiCadSchematic {
    version: u64,
    generator: Generator,
    uuid: Uuid,
    paper: String,
    _elements: Vec<Element>,
}

fn mils_to_mm(x: i32) -> f64 {
    fmils_to_mm(f64::from(x))
}
fn fmils_to_mm(x: f64) -> f64 {
    f64::from(x) / 1000.0 * 25.4
}

fn map_pin_kind(kind: epin::PinKind) -> KPinKind {
    match kind {
        PinKind::Input => KPinKind::input,
        PinKind::InputInverted => KPinKind::input,
        PinKind::Output => KPinKind::output,
        PinKind::TriState => KPinKind::tri_state,
        PinKind::Passive => KPinKind::passive,
        PinKind::PassivePos => KPinKind::passive,
        PinKind::PassiveNeg => KPinKind::passive,
        PinKind::PowerSink => KPinKind::power_in,
        PinKind::PowerSource => KPinKind::power_out,
        PinKind::PowerReturn => KPinKind::power_in,
        PinKind::OpenCollector => KPinKind::open_collector,
        PinKind::OpenEmitter => KPinKind::open_emitter,
        PinKind::NoConnect => KPinKind::no_connect,
        PinKind::Free => KPinKind::free,
        PinKind::Unspecified => KPinKind::unspecified,
    }
}

const PIN_LENGTH_MILS: i32 = 200;

fn map_pins_to_kicad(
    outline: &[Glyph],
    hide_pin_designators: bool,
    rpins: &BTreeMap<u64, EPin>,
) -> Vec<pins> {
    if outline.len() == 0 {
        return vec![];
    }
    let mut ret = vec![];
    if let Glyph::OutlineRect(r) = &outline[0] {
        if r.is_empty() {
            return vec![];
        }
        for (num, p) in rpins {
            let at = match p.location.edge {
                EdgeLocation::North => {
                    let pn_x = mils_to_mm(p.location.offset);
                    let pn_y = mils_to_mm(r.p1.y);
                    (pn_x, pn_y, 90.0)
                }
                EdgeLocation::West => {
                    let pn_x = mils_to_mm(r.p0.x - PIN_LENGTH_MILS);
                    let pn_y = mils_to_mm(p.location.offset);
                    (pn_x, pn_y, 0.0)
                }
                EdgeLocation::East => {
                    let pn_x = mils_to_mm(r.p1.x + PIN_LENGTH_MILS);
                    let pn_y = mils_to_mm(p.location.offset);
                    (pn_x, pn_y, 180.0)
                }
                EdgeLocation::South => {
                    let pn_x = mils_to_mm(p.location.offset);
                    let pn_y = mils_to_mm(r.p0.y - PIN_LENGTH_MILS);
                    (pn_x, pn_y, 90.0)
                }
            };
            println!("Pin {:?} rect {:?} at {:?}", p, r, at);
            ret.push(pins::pin {
                _kind: map_pin_kind(p.kind),
                _appears: PinAppearance::line, // TODO - add more details here
                at,
                length: mils_to_mm(PIN_LENGTH_MILS),
                _hide: None,
                name: (p.name.clone(), make_font_size(1.27)),
                number: (format!("{}", num), make_font_size(1.27)),
            })
        }
    }
    ret
}

fn map_glyphs_to_shapes(glyphs: &[Glyph], hide_outline: bool) -> Vec<glyph> {
    let mut ret = vec![];
    for x in glyphs {
        match x {
            Glyph::OutlineRect(r) => {
                if !hide_outline {
                    let start_x = mils_to_mm(r.p0.x).min(mils_to_mm(r.p1.x));
                    let stop_x = mils_to_mm(r.p0.x).max(mils_to_mm(r.p1.x));
                    let start_y = mils_to_mm(r.p0.y).min(mils_to_mm(r.p1.y));
                    let stop_y = mils_to_mm(r.p0.y).max(mils_to_mm(r.p1.y));
                    ret.push(glyph::rectangle {
                        start: (start_x, start_y),
                        end: (stop_x, stop_y),
                        _visuals: vec![
                            Visual::stroke(crate::make_stroke_width(0.254)),
                            Visual::fill(Fill(FillType::background)),
                        ],
                    });
                }
            }
            Glyph::Line(l) => ret.push(glyph::polyline {
                pts: vec![
                    xy(mils_to_mm(l.p0.x), mils_to_mm(l.p0.y)),
                    xy(mils_to_mm(l.p1.x), mils_to_mm(l.p1.y)),
                ],
                _visuals: vec![
                    Visual::stroke(crate::make_stroke_width(0.0)),
                    Visual::fill(Fill(FillType::none)),
                ],
            }),
            Glyph::Text(t) => ret.push(glyph::text {
                _text: t.text.clone(),
                at: (mils_to_mm(t.p0.x), mils_to_mm(t.p0.y), 0.0),
                _effects: Effects(vec![
                    Effect::font(vec![FontDetail::size(1.27, 1.27)]),
                    Effect::justify(match t.justify {
                        TextJustification::BottomLeft => {
                            vec![Justification::bottom, Justification::left]
                        }
                        TextJustification::BottomRight => {
                            vec![Justification::bottom, Justification::right]
                        }
                        TextJustification::TopLeft => {
                            vec![Justification::top, Justification::left]
                        }
                        TextJustification::TopRight => {
                            vec![Justification::top, Justification::right]
                        }
                        TextJustification::MiddleLeft => {
                            vec![Justification::left]
                        }
                        TextJustification::MiddleRight => {
                            vec![Justification::right]
                        }
                    }),
                ]),
            }),
            Glyph::Arc(_) => {}
            Glyph::Circle(c) => ret.push(glyph::circle {
                center: (mils_to_mm(c.p0.x), mils_to_mm(c.p0.y)),
                radius: fmils_to_mm(c.radius),
                _visuals: vec![
                    Visual::stroke(crate::make_stroke_width(0.0)),
                    Visual::fill(Fill(FillType::none)),
                ],
            }),
        }
    }
    ret
}

fn map_part_to_library_symbols(instance: &PartInstance, layout: &SchematicLayout) -> LibrarySymbol {
    let part = get_details_from_instance(instance, layout);
    LibrarySymbol {
        _name: instance.id.clone(),
        _pin_numbers: if !part.hide_pin_designators {
            None
        } else {
            Some(PinNumberOption {
                _hide: PinHide::hide,
            })
        },
        in_bom: true,
        on_board: true,
        _properties: vec![ // TODO - add these later
        ],
        _shape: shape {
            _name: format!("{}_0_1", instance.id.clone()),
            _elements: map_glyphs_to_shapes(&part.outline, part.hide_part_outline),
        },
        _pinout: pinout {
            _name: format!("{}_1_1", instance.id.clone()),
            _elements: map_pins_to_kicad(&part.outline, part.hide_pin_designators, &part.pins),
        },
    }
}

fn map_part_to_element(part: &PartInstance, layout: &SchematicLayout) -> Element {
    let details = get_details_from_instance(part, layout);
    let l = layout.part(&part.id);
    let rotation = match l.rotation {
        SchematicRotation::Vertical => 90.0,
        SchematicRotation::Horizontal => 0.0,
    };
    let location = (mils_to_mm(l.center.0), -mils_to_mm(l.center.1), rotation);
    Element::symbol {
        lib_id: part.id.clone(),
        at: location,
        unit: 1,
        in_bom: true,
        on_board: true,
        _auto: None,
        uuid: Uuid::new_v4(),
        _properties: vec![],
        _pin: details
            .pins
            .iter()
            .map(|(number, _epin)| PinMap {
                _number: format!("{}", number),
                uuid: Uuid::new_v4(),
            })
            .collect(),
    }
}

pub fn write_circuit_to_kicad6(circuit: &Circuit, layout: &SchematicLayout, name: &str) {
    let mut schematic = KiCadSchematic {
        version: 20210621,
        generator: Generator::eeschema,
        uuid: Uuid::new_v4(),
        paper: "A4".to_string(),
        _elements: vec![],
    };
    let lib = circuit
        .nodes
        .iter()
        .map(|part| map_part_to_library_symbols(part, layout))
        .collect::<Vec<_>>();
    let mut instances = circuit
        .nodes
        .iter()
        .map(|part| map_part_to_element(part, layout))
        .collect::<Vec<_>>();
    let mut lines = vec![];
    let mut junctions = vec![];
    for net in &circuit.nets {
        let ports = net
            .pins
            .iter()
            .map(|x| get_pin_net_location(&circuit, layout, x))
            .map(|x| (x.0, x.1))
            .collect::<Vec<_>>();
        let mut net_layout = layout.net(&net.name);
        if net_layout.len() == 0 {
            net_layout = make_rat_layout(ports.len());
        }
        let mut pos = (0, 0);
        for cmd in net_layout {
            match cmd {
                NetLayoutCmd::MoveToPort(n) => {
                    pos = ports[n - 1];
                }
                NetLayoutCmd::LineToPort(n) => {
                    lines.push((pos, ports[n - 1]));
                    pos = ports[n - 1];
                }
                NetLayoutCmd::MoveToCoords(x, y) => {
                    pos = (x, -y);
                }
                NetLayoutCmd::LineToCoords(x, y) => {
                    lines.push((pos, (x, -y)));
                    pos = (x, -y);
                }
                NetLayoutCmd::Junction => {
                    junctions.push(pos);
                }
            }
        }
    }
    let mut junctions = junctions
        .iter()
        .map(|x| Element::junction {
            at: (mils_to_mm(x.0), mils_to_mm(x.1)),
            diameter: 0.0,
            color: (0.0, 0.0, 0.0, 0.0),
        })
        .collect::<Vec<_>>();
    let mut wires = lines
        .iter()
        .map(|x| Element::wire {
            pts: vec![
                xy(mils_to_mm(x.0 .0), mils_to_mm(x.0 .1)),
                xy(mils_to_mm(x.1 .0), mils_to_mm(x.1 .1)),
            ],
            stroke: vec![
                StrokeDetails::width(0.0),
                StrokeDetails::kind(StrokeKind::solid),
                StrokeDetails::color(0.0, 0.0, 0.0, 0.0),
            ],
            uuid: Uuid::new_v4(),
        })
        .collect::<Vec<_>>();
    schematic._elements.push(Element::lib_symbols(lib));
    schematic._elements.append(&mut instances);
    schematic._elements.append(&mut junctions);
    schematic._elements.append(&mut wires);
    std::fs::write(name, to_s_string(&schematic).unwrap()).expect("Unable to write to file");
}
