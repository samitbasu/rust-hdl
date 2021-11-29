use crate::widgets::dff::DFF;
use crate::core::prelude::*;

#[derive(LogicBlock)]
pub struct PulseWidthModulator<const N: usize> {
    pub enable: Signal<In, Bit>,
    pub threshold: Signal<In, Bits<N>>,
    pub clock: Signal<In, Clock>,
    pub active: Signal<Out, Bit>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Default for PulseWidthModulator<N> {
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

impl<const N: usize> Logic for PulseWidthModulator<N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.counter.clk.next = self.clock.val();
        self.counter.d.next = self.counter.q.val() + 1_u32;
        self.active.next = self.enable.val() & (self.counter.q.val() < self.threshold.val());
    }
}
