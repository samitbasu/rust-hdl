use crate::dff::DFF;
use rust_hdl_core::bits::{bit_cast, Bit, Bits};
use rust_hdl_core::clock::Clock;
use rust_hdl_core::constant::Constant;
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::logic::Logic;
use rust_hdl_core::signal::Signal;
use rust_hdl_macros::{hdl_gen, LogicBlock};

#[derive(Clone, Debug, LogicBlock)]
pub struct Shot<const N: usize> {
    pub trigger: Signal<In, Bit>,
    pub active: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    duration: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
    state: DFF<Bit>,
}

impl<const N: usize> Shot<N> {
    pub fn new(duration: u64) -> Self {
        Self {
            trigger: Signal::default(),
            active: Signal::new_with_default(false),
            clock: Signal::default(),
            duration: Constant::new(duration.into()),
            counter: DFF::new(0_u32.into()),
            state: DFF::new(false),
        }
    }
}

impl<const N: usize> Logic for Shot<N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.counter.clk.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.counter.d.next = self.counter.q.val() + bit_cast::<N, 1>(self.state.q.val().into());
        self.state.d.next = self.state.q.val();
        if !self.state.q.val() && self.trigger.val() {
            self.state.d.next = true;
            self.counter.d.next = 0_u32.into();
        }
        if self.state.q.val() && (self.counter.q.val() == self.duration.val()) {
            self.state.d.next = false;
        }
        self.active.next = self.state.q.val();
    }
}
