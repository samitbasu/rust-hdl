use crate::synth::Synth;
use crate::logic::Logic;
use crate::block::Block;
use crate::probe::Probe;
use crate::atom::{Atom, AtomKind};

#[derive(Copy, Clone, Debug)]
pub struct Constant<T: Synth> {
    pub val: T,
}

impl<T: Synth> Constant<T> {
    pub fn new(val: T) -> Constant<T> {
        Constant { val }
    }
}

impl<T: Synth> Logic for Constant<T> {
    fn update(&mut self) {
    }

    fn connect(&mut self) {
    }
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
}

impl<T: Synth> Block for Constant<T> {
    fn connect_all(&mut self) {
    }

    fn update_all(&mut self) {
    }

    fn has_changed(&self) -> bool {
        false
    }

    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        probe.visit_atom(name, self);
    }
}