use crate::synth::Synth;
use crate::direction::{Direction, In, Out};
use crate::atom::Atom;
use crate::logic::Logic;
use crate::clock::Clock;
use crate::block::Block;
use crate::visitor::Visitor;
use crate::visitor_mut::VisitorMut;
use crate::scoped_visitor::ScopedVisitor;

#[derive(Copy, Clone, Debug)]
pub struct Signal<D: Direction, T: Synth> {
    pub next: T,
    pub val: T,
    prev: T,
    changed: bool,
    claimed: bool,
    dir: std::marker::PhantomData<D>,
}

impl<D: Direction, T: Synth> Atom for Signal<D, T> {
    #[inline(always)]
    fn bits(&self) -> usize {
        T::BITS
    }

    #[inline(always)]
    fn connected(&self) -> bool {
        self.claimed
    }

    #[inline(always)]
    fn changed(&self) -> bool {
        self.changed
    }
}

impl<D: Direction, T: Synth> Signal<D, T> {
    pub fn connect(&mut self) {
        assert!(!self.claimed);
        self.claimed = true;
    }
}

impl<D: Direction, T: Synth> Logic for Signal<D, T> {
    #[inline(always)]
    fn update(&mut self) {
        self.changed = self.val != self.next;
        if self.changed {
            self.prev = self.val;
            self.val = self.next;
        }
    }
}

impl<D: Direction, T: Synth> Block for Signal<D, T> {

    #[inline(always)]
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_atom(self);
    }

    #[inline(always)]
    fn accept_mut(&mut self, visitor: &mut dyn VisitorMut) {
        visitor.visit(self);
    }

    #[inline(always)]
    fn accept_scoped(&self, name: &str, visitor: &mut dyn ScopedVisitor) {
        visitor.visit_atom(name, self);
    }
}



impl Signal<In, Clock> {
    #[inline(always)]
    pub fn pos_edge(&self) -> bool {
        self.changed && self.val.0 && !self.prev.0
    }
    #[inline(always)]
    pub fn neg_edge(&self) -> bool {
        self.changed && !self.val.0 && self.prev.0
    }
}

impl<T: Synth> Signal<Out, T> {
    pub fn new_with_default(init: T) -> Signal<Out, T> {
        Self {
            next: T::default(),
            val: init,
            prev: init,
            changed: true,
            claimed: false,
            dir: std::marker::PhantomData,
        }
    }
}

impl<T: Synth> Signal<In, T> {
    pub fn new() -> Signal<In, T> {
        Self {
            next: T::default(),
            val: T::default(),
            prev: T::default(),
            changed: false,
            claimed: false,
            dir: std::marker::PhantomData,
        }
    }
}

