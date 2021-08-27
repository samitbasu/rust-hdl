use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast::{VerilogLink, VerilogLinkDetails, VerilogLiteral};
use crate::atom::{Atom, AtomKind};
use crate::block::Block;
use crate::clock::Clock;
use crate::constraint::{Constraint, PinConstraint, SignalType};
use crate::direction::{Direction, In, Out};
use crate::logic::{Logic, LogicLink};
use crate::prelude::InOut;
use crate::probe::Probe;
use crate::synth::{Synth, VCDValue};

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(1);

pub fn get_signal_id() -> usize {
    GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst)
}

#[derive(Clone, Debug)]
pub struct Signal<D: Direction, T: Synth> {
    pub next: T,
    val: T,
    prev: T,
    pub changed: bool,
    claimed: bool,
    id: usize,
    constraints: Vec<PinConstraint>,
    dir: std::marker::PhantomData<D>,
}

impl<T: Synth> LogicLink for Signal<In, T> {
    fn link(&mut self, other: &mut Self) {
        other.next = self.val();
    }
    fn link_hdl(&self, my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
        let details = VerilogLinkDetails {
            my_name: my_name.into(),
            owner_name: owner_name.into(),
            other_name: other_name.into(),
        };
        vec![VerilogLink::Forward(details)]
    }
    fn link_connect_source(&mut self) {
        // Does nothing
    }
    fn link_connect_dest(&mut self) {
        self.connect();
    }
}

impl<T: Synth> LogicLink for Signal<Out, T> {
    fn link(&mut self, other: &mut Self) {
        self.next = other.val();
    }
    fn link_hdl(&self, my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
        let details = VerilogLinkDetails {
            my_name: my_name.into(),
            owner_name: owner_name.into(),
            other_name: other_name.into(),
        };
        vec![VerilogLink::Backward(details)]
    }
    fn link_connect_source(&mut self) {
        self.connect();
    }
    fn link_connect_dest(&mut self) {}
}

impl<T: Synth> LogicLink for Signal<InOut, T> {
    fn link(&mut self, _other: &mut Self) {
        // Do nothing for bidirectional signals...
    }
    fn link_hdl(&self, my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
        let details = VerilogLinkDetails {
            my_name: my_name.into(),
            owner_name: owner_name.into(),
            other_name: other_name.into(),
        };
        vec![VerilogLink::Bidirectional(details)]
    }
    fn link_connect_source(&mut self) {
        self.connect();
    }
    fn link_connect_dest(&mut self) {
        self.connect();
    }
}

impl<D: Direction, T: Synth> Signal<D, T> {
    pub fn val(&self) -> T {
        self.val
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
    pub fn add_signal_type(&mut self, index: usize, signal: SignalType) {
        self.constraints.push(PinConstraint {
            index,
            constraint: Constraint::Kind(signal),
        });
    }
}

impl<D: Direction, T: Synth> Atom for Signal<D, T> {
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

    fn constraints(&self) -> Vec<PinConstraint> {
        self.constraints.clone()
    }
}

impl<D: Direction, T: Synth> Logic for Signal<D, T> {
    fn update(&mut self) {}
    fn connect(&mut self) {
        self.claimed = true;
    }
}

impl<D: Direction, T: Synth> Block for Signal<D, T> {
    fn connect_all(&mut self) {}

    fn update_all(&mut self) {
        self.changed = self.val != self.next;
        if self.changed {
            self.prev = self.val;
            self.val = self.next;
        }
    }

    fn has_changed(&self) -> bool {
        self.changed
    }

    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        probe.visit_atom(name, self);
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
            next: init,
            val: init,
            prev: init,
            changed: false,
            claimed: false,
            id: get_signal_id(),
            constraints: vec![],
            dir: PhantomData,
        }
    }
}

impl<D: Direction, T: Synth> Default for Signal<D, T> {
    fn default() -> Self {
        Self {
            next: T::default(),
            val: T::default(),
            prev: T::default(),
            changed: false,
            claimed: false,
            id: get_signal_id(),
            constraints: vec![],
            dir: PhantomData,
        }
    }
}
