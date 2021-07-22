use crate::dff::DFF;
use rust_hdl_core::prelude::*;

#[derive(LogicBlock)]
pub struct PulseWidthModulator<const N: usize, const F: u64> {
    pub enable: Signal<In, Bit>,
    pub threshold: Signal<In, Bits<N>>,
    pub clock: Signal<In, Clock<F>>,
    pub active: Signal<Out, Bit>,
    counter: DFF<Bits<N>, F>,
}

impl<const N: usize, const F: u64> Default for PulseWidthModulator<N, F> {
    fn default() -> Self {
        Self {
            enable: Signal::default(),
            threshold: Signal::default(),
            clock: Signal::default(),
            active: Signal::new_with_default(false),
            counter: DFF::new(0_usize.into()),
        }
    }
}

impl<const N: usize, const F: u64> Logic for PulseWidthModulator<N, F> {
    #[hdl_gen]
    fn update(&mut self) {
        self.counter.clk.next = self.clock.val();
        self.counter.d.next = self.counter.q.val() + 1_u32;
        self.active.next = self.enable.val() && (self.counter.q.val() < self.threshold.val());
    }
}