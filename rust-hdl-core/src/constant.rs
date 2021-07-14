use crate::atom::{Atom, AtomKind};
use crate::block::Block;
use crate::logic::Logic;
use crate::probe::Probe;
use crate::synth::{Synth, VCDValue};
use crate::ast::VerilogLiteral;

#[derive(Copy, Clone, Debug)]
pub struct Constant<T: Synth> {
    val: T,
}

impl<T: Synth> Constant<T> {
    pub fn new(val: T) -> Constant<T> {
        Constant { val }
    }
    pub fn val(&self) -> T {
        self.val
    }
}

impl<T: Synth> Logic for Constant<T> {
    fn update(&mut self) {}

    fn connect(&mut self) {}
}

impl<T: Synth> Atom for Constant<T> {
    fn bits(&self) -> usize {
        T::BITS
    }

    fn connected(&self) -> bool {
        true
    }

    fn changed(&self) -> bool {
        false
    }

    fn kind(&self) -> AtomKind {
        AtomKind::Constant
    }

    fn is_enum(&self) -> bool {
        T::ENUM_TYPE
    }

    fn name(&self, ndx: usize) -> &'static str {
        T::name(ndx)
    }

    fn type_name(&self) -> &'static str {
        T::TYPE_NAME
    }

    fn vcd(&self) -> VCDValue {
        self.val.vcd()
    }

    fn verilog(&self) -> VerilogLiteral {self.val.verilog()}

    fn id(&self) -> usize {
        0_usize
    }
}

impl<T: Synth> Block for Constant<T> {
    fn connect_all(&mut self) {}

    fn update_all(&mut self) {}

    fn has_changed(&self) -> bool {
        false
    }

    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        probe.visit_atom(name, self);
    }
}
