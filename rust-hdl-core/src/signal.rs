use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast::VerilogLiteral;
use crate::atom::{Atom, AtomKind};
use crate::block::Block;
use crate::clock::{Clock, Domain};
use crate::constraint::{Constraint, PinConstraint};
use crate::direction::{Direction, In, Out};
use crate::logic::Logic;
use crate::probe::Probe;
use crate::synth::{Synth, VCDValue};
use crate::tagged::Tagged;

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(1);

fn get_signal_id() -> usize {
    GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst)
}

#[derive(Clone, Debug)]
pub struct Signal<D: Direction, T: Synth, F: Domain> {
    pub next: Tagged<T, F>,
    val: T,
    prev: T,
    pub changed: bool,
    claimed: bool,
    id: usize,
    constraints: Vec<PinConstraint>,
    dir: std::marker::PhantomData<D>,
    domain: std::marker::PhantomData<F>,
}

impl<D: Direction, T: Synth, F: Domain> Signal<D, T, F> {
    pub fn val(&self) -> Tagged<T, F> {
        Tagged(self.val, Default::default())
    }

    pub fn add_constraint(&mut self, constraint: PinConstraint) {
        self.constraints.push(constraint);
    }

    pub fn add_location(&mut self, index: usize, location: &str) {
        self.constraints.push(PinConstraint {
            index,
            constraint: Constraint::Location(location.to_owned()),
        });
    }
}

impl<D: Direction, T: Synth, F: Domain> Atom for Signal<D, T, F> {
    fn bits(&self) -> usize {
        T::BITS
    }

    fn connected(&self) -> bool {
        self.claimed
    }

    fn changed(&self) -> bool {
        self.changed
    }

    fn kind(&self) -> AtomKind {
        D::KIND
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

    fn id(&self) -> usize {
        self.id
    }

    fn verilog(&self) -> VerilogLiteral {
        self.val.verilog()
    }

    fn constraints(&self) -> Vec<PinConstraint> {self.constraints.clone()}
}

impl<D: Direction, T: Synth, F: Domain> Logic for Signal<D, T, F> {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.claimed = true;
    }
}

impl<D: Direction, T: Synth, F: Domain> Block for Signal<D, T, F> {
    fn connect_all(&mut self) {}

    fn update_all(&mut self) {
        self.changed = self.val != self.next.0;
        if self.changed {
            self.prev = self.val;
            self.val = self.next.0;
        }
    }

    fn has_changed(&self) -> bool {
        self.changed
    }

    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        probe.visit_atom(name, self);
    }
}

impl<D: Domain> Signal<In, Clock, D> {
    #[inline(always)]
    pub fn pos_edge(&self) -> bool {
        self.changed && self.val.0 && !self.prev.0
    }
    #[inline(always)]
    pub fn neg_edge(&self) -> bool {
        self.changed && !self.val.0 && self.prev.0
    }
}

impl<T: Synth, F: Domain> Signal<Out, T, F> {
    pub fn new_with_default(init: T) -> Signal<Out, T, F> {
        Self {
            next: Tagged::default(),
            val: init,
            prev: init,
            changed: true,
            claimed: false,
            id: get_signal_id(),
            constraints: vec![],
            dir: PhantomData,
            domain: PhantomData,
        }
    }
}

impl<D: Direction, T: Synth, F: Domain> Default for Signal<D, T, F> {
    fn default() -> Self {
        Self {
            next: Tagged::default(),
            val: T::default(),
            prev: T::default(),
            changed: false,
            claimed: false,
            id: get_signal_id(),
            constraints: vec![],
            dir: PhantomData,
            domain: PhantomData,
        }
    }
}
