use crate::ast::{Verilog, VerilogLink};

pub trait Logic {
    fn update(&mut self);
    fn connect(&mut self) {}
    fn hdl(&self) -> Verilog {
        Verilog::Empty
    }
}

pub fn logic_connect_fn<L: Logic>(x: &mut L) {
    x.connect();
}

impl<L: Logic, const P: usize> Logic for [L; P] {
    fn update(&mut self) {}
}

pub trait LogicLink {
    fn link(&mut self, other: &mut Self);
    fn link_hdl(&self, my_name: &str, this: &str, that: &str) -> Vec<VerilogLink>;
}
