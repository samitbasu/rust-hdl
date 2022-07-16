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
