use std::fmt::Display;
use std::str::FromStr;

use crate::glyph::{pin, polyline, rectangle};
use crate::serde_s::to_s_string;
use serde::{ser, Serialize};
use uuid::Uuid;

pub mod serde_s;

#[derive(Debug, Clone, Serialize)]
pub enum FontDetail {
    size(f64, f64),
}

#[derive(Debug, Clone, Serialize)]
pub enum Justification {
    right,
}

#[derive(Debug, Clone, Serialize)]
pub enum Generator {
    eeschema,
}

#[derive(Debug, Clone, Serialize)]
pub enum Effect {
    font(Vec<FontDetail>),
    justify(Justification),
    hide,
}

#[derive(Debug, Clone, Serialize)]
pub enum Hint {
    id(u32),
    at(f64, f64, f64),
    effects(Vec<Effect>),
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

fn make_font_size(sze: f64) -> Hint {
    Hint::effects(vec![Effect::font(vec![FontDetail::size(sze, sze)])])
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
    pin {
        _kind: PinKind,
        _appears: PinAppearance,
        at: (f64, f64, f64),
        length: f64,
        _hide: Option<PinHide>,
        name: (String, Vec<Hint>),
        number: (String, Vec<Hint>),
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "symbol")]
pub struct shape {
    _name: String,
    _elements: Vec<glyph>,
}

#[derive(Debug, Clone, Serialize)]
pub struct property {
    _name: String,
    _value: String,
    _hints: Vec<Hint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct symbol {
    _name: String,
    in_bom: bool,
    on_board: bool,
    _properties: Vec<property>,
    _shapes: Vec<shape>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Element {
    lib_symbols(Vec<symbol>),
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
}

#[derive(Debug, Clone, Serialize)]
pub struct KiCadSchematic {
    version: u64,
    generator: Generator,
    uuid: Uuid,
    paper: String,
    _elements: Vec<Element>,
}

#[test]
fn test_schematic() {
    let y = KiCadSchematic {
        version: 20210621,
        generator: Generator::eeschema,
        uuid: Uuid::from_str("2a3d3e67-16d3-407f-b629-075136792054").unwrap(),
        paper: "A4".to_string(),
        _elements: vec![
            Element::lib_symbols(vec![symbol {
                _name: "Converter_DCDC:TMR2-2411WI".to_string(),
                in_bom: true,
                on_board: true,
                _properties: vec![
                    property {
                        _name: "Reference".to_string(),
                        _value: "U".to_string(),
                        _hints: vec![
                            Hint::id(0),
                            Hint::at(-7.62, 8.89, 0.0),
                            make_font_size(1.27),
                        ],
                    },
                    property {
                        _name: "Value".to_string(),
                        _value: "TMR2-2411WI".to_string(),
                        _hints: vec![
                            Hint::id(1),
                            Hint::at(11.43, 8.89, 0.0),
                            Hint::effects(vec![
                                Effect::font(vec![FontDetail::size(1.27, 1.27)]),
                                Effect::justify(Justification::right),
                            ]),
                        ],
                    },
                ],
                _shapes: vec![shape {
                    _name: "TMR2-2411WI_0_1".to_string(),
                    _elements: vec![
                        rectangle {
                            start: (-10.16, 7.62),
                            end: (10.16, -7.62),
                            _visuals: vec![
                                Visual::stroke(make_stroke_width(0.254)),
                                Visual::fill(Fill(FillType::background)),
                            ],
                        },
                        polyline {
                            pts: vec![xy(0.0, -5.08), xy(0.0, -6.35)],
                            _visuals: vec![
                                Visual::stroke(make_stroke_width(0.0)),
                                Visual::fill(Fill(FillType::none)),
                            ],
                        },
                        pin {
                            _kind: PinKind::power_in,
                            _appears: PinAppearance::line,
                            at: (-12.7, -5.08, 0.0),
                            length: 2.54,
                            _hide: None,
                            name: ("-VIN".to_string(), vec![make_font_size(1.27)]),
                            number: ("1".to_string(), vec![make_font_size(1.27)]),
                        },
                        pin {
                            _kind: PinKind::power_in,
                            _appears: PinAppearance::line,
                            at: (-12.7, 5.08, 0.0),
                            length: 2.54,
                            _hide: None,
                            name: ("+VIN".to_string(), vec![make_font_size(1.27)]),
                            number: ("2".to_string(), vec![make_font_size(1.27)]),
                        },
                        pin {
                            _kind: PinKind::no_connect,
                            _appears: PinAppearance::line,
                            at: (10.16, 2.54, 180.0),
                            length: 2.54,
                            _hide: Some(PinHide::hide),
                            name: ("NC".to_string(), vec![make_font_size(1.27)]),
                            number: ("7".to_string(), vec![make_font_size(1.27)]),
                        },
                    ],
                }],
            }]),
            Element::junction {
                at: (124.46, 57.15),
                diameter: 0.9144,
                color: (0.0, 0.0, 0.0, 0.0),
            },
            Element::junction {
                at: (133.35, 74.93),
                diameter: 0.0,
                color: (0.0, 0.0, 0.0, 0.0),
            },
            Element::wire {
                pts: vec![xy(105.41, 59.69), xy(113.03, 59.68)],
                stroke: vec![
                    StrokeDetails::width(0.0),
                    StrokeDetails::kind(StrokeKind::solid),
                    StrokeDetails::color(0.0, 0.0, 0.0, 0.0),
                ],
                uuid: Uuid::from_str("6222b708-5a6d-4f91-8f28-6abbc1ebfea0").unwrap(),
            },
        ],
    };
    let q = to_s_string(&y).unwrap();
    println!("{}", q);
}
