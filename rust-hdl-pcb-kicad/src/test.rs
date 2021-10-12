use uuid::Uuid;

use crate::glyph::{polyline, rectangle};
use crate::pins::pin;
use crate::serde_s::to_s_string;
use crate::{
    pinout, property, shape, xy, AutoFields, Effect, Element, Fill, FillType, Generator,
    Justification, KPinKind, KiCadSchematic, LibrarySymbol, Page, PinAppearance, PinHide, PinMap,
    StrokeDetails, StrokeKind, Visual,
};
use std::str::FromStr;

#[test]
fn test_schematic() {
    let y = KiCadSchematic {
        version: 20210621,
        generator: Generator::eeschema,
        uuid: Uuid::from_str("2a3d3e67-16d3-407f-b629-075136792054").unwrap(),
        paper: "A4".to_string(),
        _elements: vec![
            Element::lib_symbols(vec![LibrarySymbol {
                _name: "Converter_DCDC:TMR2-2411WI".to_string(),
                in_bom: true,
                on_board: true,
                _properties: vec![
                    property {
                        _name: "Reference".to_string(),
                        _value: "U".to_string(),
                        id: 0,
                        at: (-7.62, 8.89, 0.0),
                        effects: crate::make_font_size(1.27).0,
                    },
                    property {
                        _name: "Value".to_string(),
                        _value: "TMR2-2411WI".to_string(),
                        id: 1,
                        at: (11.43, 8.89, 0.0),
                        effects: {
                            let mut r = crate::make_font_size(1.27).0;
                            r.push(Effect::justify(vec![Justification::right]));
                            r
                        },
                    },
                ],
                _shape: shape {
                    _name: "TMR2-2411WI_0_1".to_string(),
                    _elements: vec![
                        rectangle {
                            start: (-10.16, 7.62),
                            end: (10.16, -7.62),
                            _visuals: vec![
                                Visual::stroke(crate::make_stroke_width(0.254)),
                                Visual::fill(Fill(FillType::background)),
                            ],
                        },
                        polyline {
                            pts: vec![xy(0.0, -5.08), xy(0.0, -6.35)],
                            _visuals: vec![
                                Visual::stroke(crate::make_stroke_width(0.0)),
                                Visual::fill(Fill(FillType::none)),
                            ],
                        },
                    ],
                },
                _pinout: pinout {
                    _name: "TMR2-2411WI_1_1".to_string(),
                    _elements: vec![
                        pin {
                            _kind: KPinKind::power_in,
                            _appears: PinAppearance::line,
                            at: (-12.7, -5.08, 0.0),
                            length: 2.54,
                            _hide: None,
                            name: ("-VIN".to_string(), crate::make_font_size(1.27)),
                            number: ("1".to_string(), crate::make_font_size(1.27)),
                        },
                        pin {
                            _kind: KPinKind::power_in,
                            _appears: PinAppearance::line,
                            at: (-12.7, 5.08, 0.0),
                            length: 2.54,
                            _hide: None,
                            name: ("+VIN".to_string(), crate::make_font_size(1.27)),
                            number: ("2".to_string(), crate::make_font_size(1.27)),
                        },
                        pin {
                            _kind: KPinKind::no_connect,
                            _appears: PinAppearance::line,
                            at: (10.16, 2.54, 180.0),
                            length: 2.54,
                            _hide: Some(PinHide::hide),
                            name: ("NC".to_string(), crate::make_font_size(1.27)),
                            number: ("7".to_string(), crate::make_font_size(1.27)),
                        },
                    ],
                },
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
            Element::symbol {
                lib_id: "Converter_DCDC:TMR2-2411WI".to_string(),
                at: (92.71, 64.77, 0.0),
                unit: 1,
                in_bom: true,
                on_board: true,
                _auto: Some(AutoFields {}),
                uuid: Uuid::from_str("33abe0c8-3486-4c7c-a2ef-f55fc4bed422").unwrap(),
                _properties: vec![property {
                    _name: "Reference".into(),
                    _value: "U1".into(),
                    id: 0,
                    at: (92.71, 52.07, 0.0),
                    effects: vec![],
                }],
                _pin: vec![
                    PinMap {
                        _number: "1".into(),
                        uuid: Uuid::from_str("6a5a1852-4c63-44e1-b56f-75954e493785").unwrap(),
                    },
                    PinMap {
                        _number: "2".into(),
                        uuid: Uuid::from_str("46a44479-2cb4-428c-8a60-e5bccebe2cab").unwrap(),
                    },
                    PinMap {
                        _number: "3".into(),
                        uuid: Uuid::from_str("593540a1-4bce-40ce-bd4b-b0b5c80ae860").unwrap(),
                    },
                    PinMap {
                        _number: "6".into(),
                        uuid: Uuid::from_str("ed5b2a22-eee0-4877-87ea-c441ec704706").unwrap(),
                    },
                    PinMap {
                        _number: "7".into(),
                        uuid: Uuid::from_str("e39a3e9e-c9fc-4b7f-88b9-cce9f4b1d655").unwrap(),
                    },
                    PinMap {
                        _number: "8".into(),
                        uuid: Uuid::from_str("3d3be133-493f-452c-a56b-f68961872065").unwrap(),
                    },
                    PinMap {
                        _number: "9".into(),
                        uuid: Uuid::from_str("9dffd0c4-c608-4a1d-9b31-9b886dcb2a90").unwrap(),
                    },
                ],
            },
            Element::sheet_instances {
                _pages: vec![Page {
                    _path: "/".to_string(),
                    page: "1".to_string(),
                }],
            },
        ],
    };
    let q = to_s_string(&y).unwrap();
    println!("{}", q);
}
