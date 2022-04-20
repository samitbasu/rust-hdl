use crate::core::prelude::*;
use crate::dff_setup;
use crate::widgets::dff::DFF;

#[derive(Clone, Debug, LogicBlock)]
pub struct Strobe<const N: usize> {
    pub enable: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Strobe<N> {
    pub fn new(frequency: u64, strobe_freq_hz: f64) -> Self {
        let clock_duration_femto = freq_hz_to_period_femto(frequency as f64);
        let strobe_interval_femto = freq_hz_to_period_femto(strobe_freq_hz);
        let interval = strobe_interval_femto / clock_duration_femto;
        let threshold = interval.round() as u64;
        assert!((threshold as u128) < (1_u128 << (N as u128)));
        assert!(threshold > 2);
        Self {
            enable: Signal::default(),
            strobe: Signal::default(),
            clock: Signal::default(),
            reset: Default::default(),
            threshold: Constant::new(threshold.into()),
            counter: Default::default(),
        }
    }
}

impl<const N: usize> Logic for Strobe<N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the counter clock to my clock
        dff_setup!(self, clock, reset, counter);
        if self.enable.val() {
            self.counter.d.next = self.counter.q.val() + 1_u32;
        }
        self.strobe.next = self.enable.val() & (self.counter.q.val() == self.threshold.val());
        if self.strobe.val() {
            self.counter.d.next = 1_u32.into();
        }
    }
}
