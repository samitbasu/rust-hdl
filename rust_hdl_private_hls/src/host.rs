use crate::bidi::{BidiBusM, BidiMaster};
use crate::bus::FIFOWriteController;
use crate::bus::{FIFOReadController, SoCBusController};
use crate::controller::BaseController;
use crate::cross_fifo::{CrossNarrow, CrossWiden};
use rust_hdl__core::prelude::*;
use rust_hdl__widgets::prelude::*;

// Creates a Host object that connects a bidirectional 8-bit
// bus to a Controller with the appropriate intermediate pieces.
#[derive(LogicBlock, Default)]
pub struct Host<const A: usize> {
    pub bidi_bus: BidiBusM<Bits<8>>,
    pub bus: SoCBusController<16, A>,
    pub sys_clock: Signal<In, Clock>,
    pub bidi_clock: Signal<In, Clock>,
    bidi_master: BidiMaster<Bits<8>>,
    bus_to_controller: CrossWiden<8, 4, 5, 16, 3, 4>,
    controller_to_bus: CrossNarrow<16, 3, 4, 8, 4, 5>,
    controller: BaseController<A>,
}

impl<const A: usize> Host<A> {
    pub fn new(order: WordOrder) -> Self {
        Self {
            bus_to_controller: CrossWiden::new(order),
            controller_to_bus: CrossNarrow::new(order),
            ..Default::default()
        }
    }
}

impl<const A: usize> Logic for Host<A> {
    #[hdl_gen]
    fn update(&mut self) {
        BidiBusM::<Bits<8>>::link(&mut self.bidi_bus, &mut self.bidi_master.bus);
        clock!(self, bidi_clock, bidi_master);
        FIFOWriteController::<Bits<8>>::join(
            &mut self.bidi_master.data_from_bus,
            &mut self.bus_to_controller.narrow_bus,
        );
        self.bus_to_controller.narrow_clock.next = self.bidi_clock.val();
        self.bus_to_controller.wide_clock.next = self.sys_clock.val();
        FIFOReadController::<Bits<8>>::join(
            &mut self.bidi_master.data_to_bus,
            &mut self.controller_to_bus.narrow_bus,
        );
        self.controller_to_bus.narrow_clock.next = self.bidi_clock.val();
        self.controller_to_bus.wide_clock.next = self.sys_clock.val();
        FIFOReadController::<Bits<16>>::join(
            &mut self.controller.from_cpu,
            &mut self.bus_to_controller.wide_bus,
        );
        FIFOWriteController::<Bits<16>>::join(
            &mut self.controller.to_cpu,
            &mut self.controller_to_bus.wide_bus,
        );
        clock!(self, sys_clock, controller);
        SoCBusController::<16, A>::link(&mut self.bus, &mut self.controller.bus);
    }
}

#[test]
fn test_host_synthesizes() {
    let mut uut = Host::<8>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("host", &vlog).unwrap();
}
