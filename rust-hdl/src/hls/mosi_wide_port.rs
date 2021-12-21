use crate::core::prelude::*;
use crate::widgets::dff::DFF;
use crate::hls::bus::SoCPortResponder;

#[derive(LogicBlock)]
pub struct MOSIWidePort<const W: usize, const D: usize> {
    pub bus: SoCPortResponder<D>,
    pub clock_out: Signal<Out, Clock>,
    pub port_out: Signal<Out, Bits<W>>,
    pub strobe_out: Signal<Out, Bit>,
    accum: DFF<Bits<W>>,
    state: DFF<Bits<W>>,
    address_active: DFF<Bit>,
    offset: Constant<Bits<W>>,
    modulo: Constant<Bits<8>>,
    count: DFF<Bits<8>>,
    strobe: DFF<Bit>,
}

impl<const W: usize, const D: usize> Default for MOSIWidePort<W, D> {
    fn default() -> Self {
        assert!(W > D);
        assert_eq!(W % D, 0);
        assert!(W / D < 256);
        Self {
            bus: Default::default(),
            clock_out: Default::default(),
            port_out: Default::default(),
            strobe_out: Default::default(),
            accum: Default::default(),
            state: Default::default(),
            address_active: Default::default(),
            offset: Constant::new(D.into()),
            modulo: Constant::new((W / D - 1).into()),
            count: Default::default(),
            strobe: Default::default(),
        }
    }
}

impl<const W: usize, const D: usize> Logic for MOSIWidePort<W, D> {
    #[hdl_gen]
    fn update(&mut self) {
        // Clock the internal flip flops
        self.accum.clk.next = self.bus.clock.val();
        self.state.clk.next = self.bus.clock.val();
        self.count.clk.next = self.bus.clock.val();
        self.strobe.clk.next = self.bus.clock.val();
        self.address_active.clk.next = self.bus.clock.val();
        self.clock_out.next = self.bus.clock.val();
        // Compute the select/enable flag
        self.address_active.d.next = self.bus.select.val();
        // Latch prevention
        self.count.d.next = self.count.q.val();
        self.accum.d.next = self.accum.q.val();
        self.state.d.next = self.state.q.val();
        self.bus.ready.next = false;
        self.strobe_out.next = self.strobe.q.val();
        self.strobe.d.next = false;
        if self.address_active.q.val() {
            self.bus.ready.next = true;
            if self.bus.strobe.val() {
                self.accum.d.next = self.accum.q.val() << self.offset.val()
                    | bit_cast::<W, D>(self.bus.from_controller.val());
                self.count.d.next = self.count.q.val() + 1_usize;
                if self.count.q.val() == self.modulo.val() {
                    self.count.d.next = 0_u8.into();
                    self.state.d.next = self.accum.q.val();
                    self.strobe.d.next = true;
                }
            }
        }
        self.port_out.next = self.accum.q.val();
        self.bus.to_controller.next = 0_usize.into();
    }
}
