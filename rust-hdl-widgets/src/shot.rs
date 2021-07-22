use crate::dff::DFF;
use rust_hdl_core::bits::{bit_cast, Bit, Bits};
use rust_hdl_core::clock::{Clock, NANOS_PER_FEMTO, freq_hz_to_period_femto};
use rust_hdl_core::constant::Constant;
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::logic::Logic;
use rust_hdl_core::signal::Signal;
use rust_hdl_macros::{hdl_gen, LogicBlock};
use std::time::Duration;

#[derive(Clone, Debug, LogicBlock)]
pub struct Shot<const N: usize, const F: u64> {
    pub trigger: Signal<In, Bit>,
    pub active: Signal<Out, Bit>,
    pub clock: Signal<In, Clock<F>>,
    duration: Constant<Bits<N>>,
    counter: DFF<Bits<N>, F>,
    state: DFF<Bit, F>,
}

impl<const N: usize, const F: u64> Shot<N, F> {
    pub fn new(duration: Duration) -> Self {
        let duration_nanos = duration.as_nanos() as f64 * NANOS_PER_FEMTO; // duration in femtos
        let clock_period_nanos = freq_hz_to_period_femto(F as f64);
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

impl<const N: usize, const F: u64> Logic for Shot<N, F> {
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
