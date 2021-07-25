use crate::dff::DFF;
use rust_hdl_core::prelude::*;

#[derive(LogicBlock)]
pub struct PulseWidthModulator<F: Domain, const N: usize> {
    pub enable: Signal<In, Bit, F>,
    pub threshold: Signal<In, Bits<N>, F>,
    pub clock: Signal<In, Clock, F>,
    pub active: Signal<Out, Bit, F>,
    counter: DFF<Bits<N>, F>,
}

impl<F: Domain, const N: usize> Default for PulseWidthModulator<F, N> {
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

impl<F: Domain, const N: usize> Logic for PulseWidthModulator<F, N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.counter.clk.next = self.clock.val();
        self.counter.d.next = self.counter.q.val() + 1_u32;
        self.active.next = self.enable.val() & (self.counter.q.val() < self.threshold.val());
    }
}
