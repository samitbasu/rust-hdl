use crate::ast::{Verilog, VerilogBlock, VerilogExpression};

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
    //    fn hdl(&self, this: VerilogExpression, that: VerilogExpression) -> Vec<VerilogBlock>;
}

#[macro_export]
macro_rules! link {
    ($from: expr, $to: expr) => {
        $from.link(&mut $to);
    };
}
