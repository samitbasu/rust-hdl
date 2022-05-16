use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::core::ast::{VerilogLink, VerilogLinkDetails, VerilogLiteral};
use crate::core::atom::{Atom, AtomKind};
use crate::core::bits::Bit;
use crate::core::block::Block;
use crate::core::clock::{Clock, Reset};
use crate::core::constraint::{Constraint, PinConstraint, SignalType};
use crate::core::direction::{Direction, In, Local, Out};
use crate::core::logic::{Logic, LogicJoin, LogicLink};
use crate::core::prelude::{InOut, TypeDescriptor};
use crate::core::probe::Probe;
use crate::core::synth::{Synth, VCDValue};

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
    tristate_is_output: bool,
    signal_is_undriven: bool,
    constraints: Vec<PinConstraint>,
    dir: std::marker::PhantomData<D>,
}

impl<T: Synth> Signal<In, T> {
    pub fn join(&mut self, other: &mut Signal<Out, T>) {
        self.next = other.val();
    }
    pub fn join_hdl(my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
        let details = VerilogLinkDetails {
            my_name: my_name.into(),
            owner_name: owner_name.into(),
            other_name: other_name.into(),
        };
        vec![VerilogLink::Backward(details)]
    }
}

impl<T: Synth> Signal<Out, T> {
    pub fn join(&mut self, other: &mut Signal<In, T>) {
        other.next = self.val();
    }
    pub fn join_hdl(my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
        let details = VerilogLinkDetails {
            my_name: my_name.into(),
            owner_name: owner_name.into(),
            other_name: other_name.into(),
        };
        vec![VerilogLink::Forward(details)]
    }
}

// Only the input signal gets connected in a JOIN
impl<T: Synth> LogicJoin for Signal<In, T> {
    fn join_connect(&mut self) {
        self.connect();
    }
}

impl<T: Synth> LogicJoin for Signal<Out, T> {}

impl<T: Synth> LogicJoin for Signal<InOut, T> {
    fn join_connect(&mut self) {
        self.connect();
    }
}

impl<T: Synth> LogicLink for Signal<In, T> {
    fn link(&mut self, other: &mut Self) {
        other.next = self.val();
    }
    fn link_hdl(my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
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
    fn link_hdl(my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
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
    fn link(&mut self, other: &mut Self) {
        // self is the outer scope, other is the inner scope
        // So if the inner scope is driven, we take it's value
        // and mark ourselves as driven.  If the inner scope is
        // not driven, we are not driven and we push our value
        self.tristate_is_output = other.tristate_is_output;
        if other.tristate_is_output {
            self.next = other.val();
        } else {
            other.next = self.val();
        }
        self.signal_is_undriven = other.signal_is_undriven;
    }
    fn link_hdl(my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
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
    pub fn add_constraint(&mut self, constraint: PinConstraint) {
        self.constraints.push(constraint);
    }

    pub fn add_location(&mut self, index: usize, location: &str) {
        assert!(index < T::BITS);
        self.constraints.push(PinConstraint {
            index,
            constraint: Constraint::Location(location.to_owned()),
        });
    }
    pub fn add_signal_type(&mut self, index: usize, signal: SignalType) {
        assert!(index < T::BITS);
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

    fn descriptor(&self) -> TypeDescriptor {
        T::descriptor()
    }

    fn vcd(&self) -> VCDValue {
        if !self.signal_is_undriven {
            self.val.vcd()
        } else {
            VCDValue::Vector(vec![vcd::Value::Z; T::BITS])
        }
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
        self.changed && self.val.clk && !self.prev.clk
    }
    #[inline(always)]
    pub fn neg_edge(&self) -> bool {
        self.changed && !self.val.clk && self.prev.clk
    }
}

impl Signal<In, Reset> {
    #[inline(always)]
    pub fn pos_edge(&self) -> bool {
        self.changed && self.val.rst && !self.prev.rst
    }
    #[inline(always)]
    pub fn neg_edge(&self) -> bool {
        self.changed && !self.val.rst && self.prev.rst
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
            tristate_is_output: false,
            signal_is_undriven: false,
            constraints: vec![],
            dir: PhantomData,
        }
    }
}

impl<D: Direction> Signal<D, Bit> {
    pub fn pin_signal(location: &str, kind: SignalType) -> Signal<D, Bit> {
        let mut ret = Signal::default();
        ret.add_location(0, location);
        ret.add_signal_type(0, kind);
        ret
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
            tristate_is_output: false,
            signal_is_undriven: false,
            constraints: vec![],
            dir: PhantomData,
        }
    }
}

impl<T: Synth> Signal<InOut, T> {
    pub fn set_tristate_is_output(&mut self, flag: bool) {
        if self.tristate_is_output != flag {
            self.changed = true;
        }
        self.tristate_is_output = flag;
        self.signal_is_undriven = !flag;
    }
    pub fn is_driving_tristate(&self) -> bool {
        self.tristate_is_output
    }
    pub fn simulate_connected_tristate(&mut self, other: &mut Self) {
        //        assert!(!(self.is_driving_tristate() & other.is_driving_tristate()));
        if self.is_driving_tristate() {
            other.next = self.val();
            self.signal_is_undriven = false;
            other.signal_is_undriven = false;
        } else if other.is_driving_tristate() {
            self.next = other.val();
            self.signal_is_undriven = false;
            other.signal_is_undriven = false;
        } else {
            self.signal_is_undriven = true;
            other.signal_is_undriven = true;
        }
    }
}

impl<T: Synth> Signal<InOut, T> {
    pub fn join(&mut self, other: &mut Signal<InOut, T>) {
        self.simulate_connected_tristate(other);
    }
    pub fn join_hdl(my_name: &str, owner_name: &str, other_name: &str) -> Vec<VerilogLink> {
        let details = VerilogLinkDetails {
            my_name: my_name.into(),
            owner_name: owner_name.into(),
            other_name: other_name.into(),
        };
        vec![VerilogLink::Bidirectional(details)]
    }
}

// For local signals only
// There should be a write before a read
//  (will that trigger latch detection if violated?)
// .val() -> next
// local signals do not generate changes.
// Need loop detection

impl<T: Synth> Signal<Local, T> {
    pub fn val(&self) -> T {
        self.next
    }
}

impl<T: Synth> Signal<In, T> {
    pub fn val(&self) -> T {
        self.val
    }
}

impl<T: Synth> Signal<Out, T> {
    pub fn val(&self) -> T {
        self.next
    }
}

impl<T: Synth> Signal<InOut, T> {
    pub fn val(&self) -> T {
        self.val
    }
}
