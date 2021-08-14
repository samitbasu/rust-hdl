use crate::dff::DFF;
use rust_hdl_core::prelude::*;

#[derive(Clone, Debug, LogicBlock)]
pub struct Strobe<const FREQ: u64, const N: usize> {
    pub enable: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const FREQ: u64, const N: usize> Strobe<FREQ, N> {
    pub fn new(strobe_freq_hz: f64) -> Self {
        let clock_duration_femto = freq_hz_to_period_femto(FREQ as f64);
        let strobe_interval_femto = freq_hz_to_period_femto(strobe_freq_hz);
        let interval = strobe_interval_femto / clock_duration_femto;
        let threshold = interval.round() as u64;
        assert!((threshold as u128) < (1_u128 << (N as u128)));
        assert!(threshold > 1);
        Self {
            enable: Signal::default(),
            strobe: Signal::default(),
            clock: Signal::default(),
            threshold: Constant::new(threshold.into()),
            counter: DFF::new(0_usize.into()),
        }
    }
}

impl<const FREQ: u64, const N: usize> Logic for Strobe<FREQ, N> {
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the counter clock to my clock
        self.counter.clk.next = self.clock.val();
        // Latch prevention
        self.counter.d.next = self.counter.q.val();
        if self.enable.val() {
            self.counter.d.next = self.counter.q.val() + 1_u32;
        }
        self.strobe.next = self.enable.val() & (self.counter.q.val() == self.threshold.val());
        if self.strobe.val() {
            self.counter.d.next = 1_u32.into();
        }
    }
}
