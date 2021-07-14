use crate::dff::DFF;
use rust_hdl_core::bits::{Bit, Bits};
use rust_hdl_core::clock::Clock;
use rust_hdl_core::constant::Constant;
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::logic::Logic;
use rust_hdl_core::signal::Signal;
use rust_hdl_macros::{hdl_gen, LogicBlock};
use std::num::Wrapping;

#[derive(Clone, Debug, LogicBlock)]
pub struct Strobe<const N: usize> {
    pub enable: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Strobe<N> {
    pub fn new(clock_freq: u64, strobe_freq: u64) -> Self {
        let incr = Wrapping(((strobe_freq << 32) / clock_freq) as u32);
        Self {
            enable: Signal::default(),
            strobe: Signal::default(),
            clock: Signal::default(),
            threshold: Constant::new(incr.into()),
            counter: DFF::new(0_usize.into()),
        }
    }
}

impl<const N: usize> Logic for Strobe<N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.counter.clk.next = self.clock.val();
        if self.enable.val() {
            self.counter.d.next = self.counter.q.val() + self.threshold.val();
        }
        self.strobe.next = self.enable.val() & (self.counter.q.val() < self.threshold.val());
    }
}
