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
pub enum Glyph {
    OutlineRect(Rect),
    Line(Line),
    Text(Text),
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
