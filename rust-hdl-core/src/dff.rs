use crate::scoped_visitor::ScopedVisitor;
use crate::visitor_mut::VisitorMut;
use crate::visitor::Visitor;
use crate::block::Block;
use crate::synth::Synth;
use crate::logic::Logic;
use crate::clock::Clock;
use crate::direction::{In, Out};
use crate::signal::Signal;

pub struct DFF<T: Synth> {
    pub d: Signal<In, T>,
    pub q: Signal<Out, T>,
    pub clk: Signal<In, Clock>,
}

impl<T: Synth> DFF<T> {
    pub fn new(init: T) -> DFF<T> {
        let mut dff = Self {
            d: Signal::new(),
            q: Signal::new_with_default(init), // This should be marked as a register, since we write to it on a clock edge
            clk: Signal::<In, Clock>::new(),
        };
        dff.q.connect();
        dff
    }
}

impl<T: Synth> Logic for DFF<T> {
    fn update(&mut self) {
        if self.clk.pos_edge() {
            self.q.next = self.d.val
        }
    }
}

impl<T: Synth> Block for DFF<T> {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit(self);
        self.d.accept(visitor);
        self.q.accept(visitor);
        self.clk.accept(visitor);
    }

    fn accept_mut(&mut self, visitor: &mut dyn VisitorMut) {
        visitor.visit(self);
        self.d.accept_mut(visitor);
        self.q.accept_mut(visitor);
        self.clk.accept_mut(visitor);
    }

    fn accept_scoped(&self, name: &str, visitor: &mut dyn ScopedVisitor) {
        visitor.visit_start_scope(name, self);
        self.d.accept_scoped("d", visitor);
        self.q.accept_scoped("q", visitor);
        self.clk.accept_scoped("clk", visitor);
        visitor.visit_end_scope(name, self);
    }
}
