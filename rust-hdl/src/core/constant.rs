use crate::core::ast::VerilogLiteral;
use crate::core::atom::{Atom, AtomKind};
use crate::core::block::Block;
use crate::core::constraint::PinConstraint;
use crate::core::logic::Logic;
use crate::core::prelude::TypeDescriptor;
use crate::core::probe::Probe;
use crate::core::signal::get_signal_id;
use crate::core::synth::{Synth, VCDValue};

#[derive(Copy, Clone, Debug)]
pub struct Constant<T: Synth> {
    val: T,
    id: usize,
}

impl<T: Synth> Constant<T> {
    pub fn new(val: T) -> Constant<T> {
        Constant {
            val,
            id: get_signal_id(),
        }
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

    fn descriptor(&self) -> TypeDescriptor {
        T::descriptor()
    }

    fn vcd(&self) -> VCDValue {
        self.val.vcd()
    }

    fn verilog(&self) -> VerilogLiteral {
        self.val.verilog()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn constraints(&self) -> Vec<PinConstraint> {
        vec![]
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
