use serde::{Deserialize, Serialize};

use crate::epin::EdgeLocation;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Into<Point> for (i32, i32) {
    fn into(self) -> Point {
        Point {
            x: self.0,
            y: self.1,
        }
    }
}

impl Point {
    pub fn fliplr(&self) -> Point {
        Point {
            x: -self.x,
            y: self.y,
        }
    }
    pub fn flipud(&self) -> Point {
        Point {
            x: self.x,
            y: -self.y,
        }
    }
    pub fn dx() -> Point {
        Point { x: 1, y: 0 }
    }
    pub fn dy() -> Point {
        Point { x: 0, y: 1 }
    }
    pub fn min(&self, other: Point) -> Point {
        Point {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }
    pub fn max(&self, other: Point) -> Point {
        Point {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
    pub fn zero() -> Point {
        Point { x: 0, y: 0 }
    }
    pub fn rot90(&self) -> Point {
        Point {
            x: -self.y,
            y: self.x,
        }
    }
}

impl std::ops::Mul<i32> for Point {
    type Output = Point;

    fn mul(self, rhs: i32) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub p0: Point,
    pub p1: Point,
}

impl Rect {
    pub fn empty() -> Self {
        Rect {
            p0: Point::zero(),
            p1: Point::zero(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.p0 == Point::zero() && self.p1 == Point::zero()
    }
    pub fn union(&self, other: Rect) -> Rect {
        Rect {
            p0: self.p0.min(other.p0),
            p1: self.p1.max(other.p1),
        }
    }
    pub fn fliplr(&self) -> Rect {
        Rect {
            p0: self.p0.fliplr().min(self.p1.fliplr()),
            p1: self.p0.fliplr().max(self.p1.fliplr()),
        }
    }
    pub fn flipud(&self) -> Rect {
        Rect {
            p0: self.p0.flipud().min(self.p1.flipud()),
            p1: self.p0.flipud().max(self.p1.flipud()),
        }
    }
    pub fn rot90(&self) -> Rect {
        Rect {
            p0: self.p0.rot90().min(self.p1.rot90()),
            p1: self.p0.rot90().max(self.p1.rot90()),
        }
    }
    pub fn width(&self) -> i32 {
        2 * i32::max(self.p1.x.abs(), self.p0.x.abs())
        //        (self.p1.x - self.p0.x).abs()
    }
    pub fn height(&self) -> i32 {
        2 * i32::max(self.p1.y.abs(), self.p0.y.abs())
        //        (self.p1.y - self.p0.y).abs()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Line {
    pub p0: Point,
    pub p1: Point,
}

impl Line {
    pub fn fliplr(&self) -> Line {
        Line {
            p0: self.p0.fliplr(),
            p1: self.p1.fliplr(),
        }
    }
    pub fn flipud(&self) -> Line {
        Line {
            p0: self.p0.flipud(),
            p1: self.p1.flipud(),
        }
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum TextJustification {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
    MiddleLeft,
    MiddleRight,
}

impl TextJustification {
    pub fn fliplr(&self) -> TextJustification {
        match self {
            TextJustification::BottomLeft => TextJustification::BottomRight,
            TextJustification::BottomRight => TextJustification::BottomLeft,
            TextJustification::TopLeft => TextJustification::TopRight,
            TextJustification::TopRight => TextJustification::TopLeft,
            TextJustification::MiddleLeft => TextJustification::MiddleRight,
            TextJustification::MiddleRight => TextJustification::MiddleLeft,
        }
    }
    pub fn flipud(&self) -> TextJustification {
        match self {
            TextJustification::BottomLeft => TextJustification::TopLeft,
            TextJustification::BottomRight => TextJustification::TopRight,
            TextJustification::TopLeft => TextJustification::BottomLeft,
            TextJustification::TopRight => TextJustification::BottomRight,
            TextJustification::MiddleLeft => TextJustification::MiddleLeft,
            TextJustification::MiddleRight => TextJustification::MiddleRight,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Text {
    pub p0: Point,
    pub text: String,
    pub justify: TextJustification,
}

impl Text {
    pub fn fliplr(&self) -> Text {
        Self {
            p0: self.p0.fliplr(),
            text: self.text.clone(),
            justify: self.justify.fliplr(),
        }
    }
    pub fn flipud(&self) -> Text {
        Self {
            p0: self.p0.flipud(),
            text: self.text.clone(),
            justify: self.justify.flipud(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Circle {
    pub p0: Point,
    pub radius: f64,
}

impl Circle {
    pub fn fliplr(&self) -> Self {
        Self {
            p0: self.p0.fliplr(),
            radius: self.radius,
        }
    }
    pub fn flipud(&self) -> Self {
        Self {
            p0: self.p0.flipud(),
            radius: self.radius,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Arc {
    pub p0: Point,
    pub radius: f64,
    pub start_angle: f64,
    pub sweep_angle: f64,
}

impl Arc {
    pub fn fliplr(&self) -> Self {
        Self {
            p0: self.p0.fliplr(),
            radius: self.radius,
            start_angle: 180.0 - self.start_angle,
            sweep_angle: -self.sweep_angle,
        }
    }
    pub fn flipud(&self) -> Self {
        Self {
            p0: self.p0.flipud(),
            radius: self.radius,
            start_angle: -self.start_angle,
            sweep_angle: -self.sweep_angle,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Glyph {
    OutlineRect(Rect),
    Line(Line),
    Text(Text),
    Arc(Arc),
    Circle(Circle),
}

impl Glyph {
    pub fn fliplr(&self) -> Glyph {
        match self {
            Glyph::OutlineRect(r) => Glyph::OutlineRect(r.fliplr()),
            Glyph::Line(l) => Glyph::Line(l.fliplr()),
            Glyph::Text(t) => Glyph::Text(t.fliplr()),
            Glyph::Arc(a) => Glyph::Arc(a.fliplr()),
            Glyph::Circle(c) => Glyph::Circle(c.fliplr()),
        }
    }
    pub fn flipud(&self) -> Glyph {
        match self {
            Glyph::OutlineRect(r) => Glyph::OutlineRect(r.flipud()),
            Glyph::Line(l) => Glyph::Line(l.flipud()),
            Glyph::Text(t) => Glyph::Text(t.flipud()),
            Glyph::Arc(a) => Glyph::Arc(a.flipud()),
            Glyph::Circle(c) => Glyph::Circle(c.flipud()),
        }
    }

    // These are fairly crude approximations...
    pub fn estimate_bounding_box(&self) -> Rect {
        match self {
            Glyph::OutlineRect(r) => Rect {
                p0: r.p0 + Point::dx() * (-200) + Point::dy() * (-200),
                p1: r.p1 + Point::dx() * 200 + Point::dy() * 200,
            },
            Glyph::Line(l) => Rect {
                p0: l.p0.min(l.p1),
                p1: l.p0.max(l.p1),
            },
            Glyph::Text(t) => {
                let mut tx = (t.text.len() * 55) as i32;
                let mut ty = 85;
                let mut dy = 0;
                match t.justify {
                    TextJustification::BottomLeft => {}
                    TextJustification::BottomRight => {
                        tx = -tx;
                    }
                    TextJustification::TopLeft => {
                        ty = -ty;
                    }
                    TextJustification::TopRight => {
                        ty = -ty;
                        tx = -tx;
                    }
                    TextJustification::MiddleLeft => {
                        dy = -42;
                    }
                    TextJustification::MiddleRight => {
                        dy = -42;
                        tx = -tx;
                    }
                }
                let p0 = t.p0 + Point::dy() * dy;
                let p1 = t.p0 + Point::dx() * tx + Point::dy() * ty;
                Rect {
                    p0: p0.min(p1),
                    p1: p0.max(p1),
                }
            }
            Glyph::Arc(a) => Rect {
                p0: a.p0 + Point::dx() * (-a.radius as i32) + Point::dy() * (-a.radius as i32),
                p1: a.p0 + Point::dx() * (a.radius as i32) + Point::dy() * (a.radius as i32),
            },
            Glyph::Circle(a) => Rect {
                p0: a.p0 + Point::dx() * (-a.radius as i32) + Point::dy() * (-a.radius as i32),
                p1: a.p0 + Point::dx() * (a.radius as i32) + Point::dy() * (a.radius as i32),
            },
        }
    }
}

pub fn estimate_bounding_box(glyphs: &Vec<Glyph>) -> Rect {
    if glyphs.len() == 0 {
        return Rect {
            p0: Point::zero(),
            p1: Point::zero(),
        };
    }
    let mut bbox = glyphs[0].estimate_bounding_box();
    for glyph in glyphs {
        bbox = bbox.union(glyph.estimate_bounding_box());
    }
    bbox
}

pub fn make_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Glyph {
    Glyph::Line(Line {
        p0: Point { x: x0, y: y0 },
        p1: Point { x: x1, y: y1 },
    })
}

pub fn make_ic_body(x0: i32, y0: i32, x1: i32, y1: i32) -> Glyph {
    Glyph::OutlineRect(Rect {
        p0: Point { x: x0, y: y0 },
        p1: Point { x: x1, y: y1 },
    })
}

pub fn make_label(x0: i32, y0: i32, msg: &str, justify: TextJustification) -> Glyph {
    Glyph::Text(Text {
        p0: Point { x: x0, y: y0 },
        text: msg.into(),
        justify,
    })
}

pub fn make_arc(x0: i32, y0: i32, radius: f64, start_angle: f64, sweep_angle: f64) -> Glyph {
    Glyph::Arc(Arc {
        p0: Point { x: x0, y: y0 },
        radius,
        start_angle,
        sweep_angle,
    })
}
