use crate::block::Block;
use crate::clock::Clock;
use crate::direction::{In, Out};
use crate::logic::Logic;
use crate::probe::Probe;
use crate::signal::Signal;
use crate::synth::Synth;

#[derive(Clone, Debug)]
pub struct DFF<T: Synth> {
    pub d: Signal<In, T>,
    pub q: Signal<Out, T>,
    pub clk: Signal<In, Clock>,
}

impl<T: Synth> DFF<T> {
    pub fn new(init: T) -> DFF<T> {
        Self {
            d: Signal::default(),
            q: Signal::new_with_default(init), // This should be marked as a register, since we write to it on a clock edge
            clk: Signal::default(),
        }
    }
}

impl<T: Synth> Logic for DFF<T> {
    fn update(&mut self) {
        if self.clk.pos_edge() {
            self.q.next = self.d.val
        }
    }
    fn connect(&mut self) {
        self.q.connect();
    }
}

impl<T: Synth> Block for DFF<T> {
    fn connect_all(&mut self) {
        self.connect();
        self.d.connect_all();
        self.q.connect_all();
        self.clk.connect_all();
    }

    fn update_all(&mut self) {
        self.update();
        self.d.update_all();
        self.q.update_all();
        self.clk.update_all();
    }

    fn has_changed(&self) -> bool {
        self.d.changed || self.q.changed || self.clk.changed
    }

    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        probe.visit_start_scope(name, self);
        self.d.accept("d", probe);
        self.q.accept("q", probe);
        self.clk.accept("clk", probe);
        probe.visit_end_scope(name, self);
    }
}
