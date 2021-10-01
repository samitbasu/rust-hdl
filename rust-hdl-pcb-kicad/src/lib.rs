#![allow(non_camel_case_types)]

use rust_hdl_pcb_core::prelude::*;
use std::fmt::Display;
use std::str::FromStr;

use serde::{ser, Serialize};
use uuid::Uuid;

use crate::glyph::{polyline, rectangle};
use crate::pins::pin;
use crate::serde_s::to_s_string;

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
pub enum PinKind {
    power_in,
    input,
    power_out,
    no_connect,
    output,
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
        _kind: PinKind,
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

fn map_glyphs_to_shapes(glyphs: &[Glyph], hide_outline: bool) -> Vec<glyph> {
    let mut ret = vec![];
    for x in &glyphs {
        match x {
            Glyph::OutlineRect(r) => {
                if !hide_outline {
                    ret.push(glyph::rectangle {
                        start: (mils_to_mm(r.p0.x), -mils_to_mm(-r.p0.y)),
                        end: (mils_to_mm(r.p1.x), -mils_to_mm(-r.p1.y)),
                        _visuals: vec![
                            Visual::stroke(crate::make_stroke_width(0.254)),
                            Visual::fill(Fill(FillType::background)),
                        ],
                    });
                }
            }
            Glyph::Line(l) => ret.push(glyph::polyline {
                pts: vec![
                    xy(mils_to_mm(l.p0.x), mils_to_mm(-l.p0.y)),
                    xy(mils_to_mm(l.p1.x), mils_to_mm(-l.p1.y)),
                ],
                _visuals: vec![
                    Visual::stroke(crate::make_stroke_width(0.0)),
                    Visual::fill(Fill(FillType::none)),
                ],
            }),
            Glyph::Text(t) => ret.push(glyph::text {
                _text: t.text.clone(),
                at: (mils_to_mm(t.p0.x), mils_to_mm(-t.p1.y), 0.0),
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
            Glyph::Circle(c) => glyph::circle {
                center: (mils_to_mm(c.p0.x), mils_to_mm(-c.p0.y)),
                radius: fmils_to_mm(c.radius),
                _visuals: vec![
                    Visual::stroke(crate::make_stroke_width(0.0)),
                    Visual::fill(Fill(FillType::none)),
                ],
            },
        }
    }
    ret
}

fn map_part_to_library_symbols(part: &PartDetails) -> LibrarySymbol {
    LibrarySymbol {
        _name: part.label.to_string(),
        in_bom: true,
        on_board: true,
        _properties: vec![ // TODO - add these later
        ],
        _shape: shape {
            _name: part.label.to_string(),
            _elements: map_glyphs_to_shapes(&part.outline, part.hide_part_outline),
        },
        _pinout: pinout {
            _name: part.label.to_string(),
            _elements: vec![],
        },
    }
    // The first symbol represents the
}

pub fn write_circuit_to_kicad6(circuit: &Circuit, layout: &SchematicLayout, name: &str) {
    let mut schematic = KiCadSchematic {
        version: 20210621,
        generator: Generator::eeschema,
        uuid: Uuid::new_v4(),
        paper: "A4".to_string(),
        _elements: vec![],
    };
    for part in &circuit.nodes {
        let part = get_details_from_instance(part, layout);
        let symbols = map_part_to_library_symbols(&part);
    }
}
