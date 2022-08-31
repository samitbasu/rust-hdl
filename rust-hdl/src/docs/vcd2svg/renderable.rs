use num_bigint::BigInt;
use std::clone::Clone;

pub trait Renderable {
    fn render(&self) -> String;
}

impl Renderable for BigInt {
    fn render(&self) -> String {
        format!("0h{:x}", self)
    }
}

impl Renderable for String {
    fn render(&self) -> String {
        self.clone()
    }
}

impl Renderable for bool {
    fn render(&self) -> String {
        if *self {
            "1".to_string()
        } else {
            "0".to_string()
        }
    }
}
