use crate::core::ast::{Verilog, VerilogLink};

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

/*
 A link is always
 In --> In
 Out --> Out

 So if we have a piece of logic with:
 > In (A)   ---- Link ----    > In(A)
 < Out (B)  ---- Link ----    < Out(B)

 We want the connections for the internal parts to be
 handled automatically by RustHDL.  So Dest In should always be
 driven, and source Out should always be driven.

 When externally connected, we assume that the situationis

 External              Internal
 >Out (A) --- Link --->In(A)
 <In (A)  --- Link ---<Out(A)
*/
pub trait LogicLink {
    fn link(&mut self, other: &mut Self);
    fn link_hdl(&self, my_name: &str, this: &str, that: &str) -> Vec<VerilogLink>;
    fn link_connect_source(&mut self);
    fn link_connect_dest(&mut self);
}

pub fn logic_connect_link_fn<L: LogicLink>(source: &mut L, dest: &mut L) {
    source.link_connect_source();
    dest.link_connect_dest();
}

pub trait LogicJoin {
    fn join_connect(&mut self) {}
    fn join_hdl(&self, _my_name: &str, _this: &str, _that: &str) -> Vec<VerilogLink> {
        vec![]
    }
}

pub fn logic_connect_join_fn<L: LogicJoin, K: LogicJoin>(source: &mut L, dest: &mut K) {
    source.join_connect();
    dest.join_connect();
}
