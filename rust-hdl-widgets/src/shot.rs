use crate::dff::DFF;
use rust_hdl_core::prelude::*;
use rust_hdl_macros::{hdl_gen, LogicBlock};
use std::time::Duration;

#[derive(Clone, Debug, LogicBlock)]
pub struct Shot<F: Domain, const N: usize> {
    pub trigger: Signal<In, Bit, F>,
    pub active: Signal<Out, Bit, F>,
    pub clock: Signal<In, Clock, F>,
    duration: Constant<Bits<N>>,
    counter: DFF<Bits<N>, F>,
    state: DFF<Bit, F>,
}

impl<F: Domain, const N: usize> Shot<F, N> {
    pub fn new(duration: Duration) -> Self {
        let duration_nanos = duration.as_nanos() as f64 * NANOS_PER_FEMTO; // duration in femtos
        let clock_period_nanos = freq_hz_to_period_femto(F::FREQ as f64);
        let clocks = (duration_nanos / clock_period_nanos).floor() as u64;
        assert!(clocks < (1_u64 << N));
        Self {
            trigger: Signal::default(),
            active: Signal::new_with_default(false),
            clock: Signal::default(),
            duration: Constant::new(clocks.into()),
            counter: DFF::new(0_u32.into()),
            state: DFF::new(false),
        }
    }
}

impl<F: Domain, const N: usize> Logic for Shot<F, N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.counter.clk.next = self.clock.val();
        self.state.clk.next = self.clock.val();
        self.counter.d.next = self.counter.q.val();
        if self.state.q.val().raw() {
            self.counter.d.next = self.counter.q.val() + 1_u32;
        }
        self.state.d.next = self.state.q.val();
        if !self.state.q.val().raw() && self.trigger.val().raw() {
            self.state.d.next = true.into();
            self.counter.d.next = 0_u32.into();
        }
        if self.state.q.val().raw() && (self.counter.q.val() == self.duration.val()) {
            self.state.d.next = false.into();
        }
        self.active.next = self.state.q.val();
    }
}
