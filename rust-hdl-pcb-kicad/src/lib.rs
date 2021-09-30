use std::fmt::Display;
use std::str::FromStr;

use crate::glyph::{polyline, rectangle};
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
pub enum Visual {
    fill(Fill),
    stroke(Width),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "width")]
pub struct Width(f64);

#[derive(Debug, Clone, Serialize)]
pub enum FillType {
    background,
}

#[derive(Debug, Clone, Serialize)]
pub struct xy(f64, f64);

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "type")]
pub struct Fill(Option<FillType>);

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
pub struct KiCadSchematic {
    version: u64,
    generator: Generator,
    uuid: Uuid,
    paper: String,
    lib_symbols: Vec<symbol>,
}

#[test]
fn test_schematic() {
    let y = KiCadSchematic {
        version: 20210621,
        generator: Generator::eeschema,
        uuid: Uuid::from_str("2a3d3e67-16d3-407f-b629-075136792054").unwrap(),
        paper: "A4".to_string(),
        lib_symbols: vec![symbol {
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
                        Hint::effects(vec![Effect::font(vec![FontDetail::size(1.27, 1.27)])]),
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
                            Visual::stroke(Width(0.254)),
                            Visual::fill(Fill(Some(FillType::background))),
                        ],
                    },
                    polyline {
                        pts: vec![xy(0.0, -5.08), xy(0.0, -6.35)],
                        _visuals: vec![Visual::stroke(Width(0.0)), Visual::fill(Fill(None))],
                    },
                ],
            }],
        }],
    };
    let q = to_s_string(&y).unwrap();
    println!("{}", q);
}
