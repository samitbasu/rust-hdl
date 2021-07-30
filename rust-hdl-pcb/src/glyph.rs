use crate::epin::{PinLocation, EdgeLocation};

#[derive(Clone, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug)]
pub struct Rect {
    pub p0: Point,
    pub p1: Point,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub p0: Point,
    pub p1: Point,
}

#[derive(Clone, Debug)]
pub struct Text {
    pub p0: Point,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Pin {
    pub p0: Point,
    pub location: EdgeLocation,
    pub length: i32,
}

#[derive(Clone, Debug)]
pub struct Arc {
    pub p0: Point,
    pub radius: f64,
    pub start_angle: f64,
    pub sweep_angle: f64,
}

#[derive(Clone, Debug)]
pub enum Glyph {
    OutlineRect(Rect),
    Line(Line),
    Text(Text),
    Pin(Pin),
    Arc(Arc),
}

pub fn make_pin(x0: i32, y0: i32, location: EdgeLocation, len: i32) -> Glyph {
    Glyph::Pin(Pin {
        p0: Point {
            x: x0,
            y: y0,
        },
        location,
        length: len
    })
}

pub fn make_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Glyph {
    Glyph::Line(Line {
        p0: Point{
            x: x0,
            y: y0,
        },
        p1: Point {
            x: x1,
            y: y1,
        }
    })
}

pub fn make_ic_body(x0: i32, y0: i32, x1: i32, y1: i32) -> Glyph {
    Glyph::OutlineRect(Rect{
        p0: Point {
            x: x0,
            y: y0
        },
        p1: Point {
            x: x1,
            y: y1
        }
    })
}

pub fn make_label(x0: i32, y0: i32, msg: &str) -> Glyph {
    Glyph::Text(Text {
        p0: Point {
            x: x0,
            y: y0
        },
        text: msg.into()
    })
}

pub fn make_arc(x0: i32, y0: i32, radius: f64, start_angle: f64, sweep_angle: f64) -> Glyph {
    Glyph::Arc(Arc {
        p0: Point {
            x: x0,
            y: y0
        },
        radius,
        start_angle,
        sweep_angle
    })
}