use rust_hdl_core::bits::{Bit, Bits};
use rust_hdl_core::clock::Clock;
use rust_hdl_core::constant::Constant;
use rust_hdl_core::dff::DFF;
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::logic::Logic;
use rust_hdl_core::signal::Signal;
use rust_hdl_macros::{hdl_gen, LogicBlock};

#[derive(Clone, Debug, LogicBlock)]
pub struct Strobe<const N: usize> {
    pub enable: Signal<In, Bit>,
    pub strobe: Signal<Out, Bit>,
    pub clock: Signal<In, Clock>,
    pub strobe_incr: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Default for Strobe<N> {
    fn default() -> Self {
        Self {
            enable: Signal::default(),
            strobe: Signal::<Out, Bit>::new_with_default(false),
            clock: Signal::default(),
            strobe_incr: Constant::new(1_usize.into()),
            counter: DFF::new(0_usize.into()),
        }
    }
}

impl<const N: usize> Logic for Strobe<N> {
    #[hdl_gen]
    fn update(&mut self) {
        self.counter.clk.next = self.clock.val;
        if self.enable.val {
            self.counter.d.next = self.counter.q.val + self.strobe_incr.val;
        }
        self.strobe.next = self.enable.val & !self.counter.q.val.any();
    }
}
/*
    fn connect(&mut self) {
        self.counter.clk.connect();
        self.strobe.connect();
        self.counter.d.connect();
    }
*/
