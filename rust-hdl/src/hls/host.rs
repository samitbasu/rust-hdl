use crate::core::prelude::*;
use crate::hls::bidi::{BidiBusM, BidiMaster};
use crate::hls::bus::FIFOWriteController;
use crate::hls::bus::{FIFOReadController, SoCBusController};
use crate::hls::controller::BaseController;
use crate::hls::cross_fifo::{CrossNarrow, CrossWiden};
use crate::widgets::prelude::*;
use crate::widgets::reset_sync::ResetSynchronizer;

// Creates a Host object that connects a bidirectional 8-bit
// bus to a Controller with the appropriate intermediate pieces.
#[derive(LogicBlock, Default)]
pub struct Host<const A: usize> {
    pub bidi_bus: BidiBusM<Bits<8>>,
    pub bus: SoCBusController<16, A>,
    pub sys_clock: Signal<In, Clock>,
    pub bidi_clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    bidi_master: BidiMaster<Bits<8>>,
    bus_to_controller: CrossWiden<8, 4, 5, 16, 3, 4>,
    controller_to_bus: CrossNarrow<16, 3, 4, 8, 4, 5>,
    controller: BaseController<A>,
    sys_reset: Signal<Local, Reset>,
    bidi_reset: Signal<Local, Reset>,
    sys_reset_sync: ResetSynchronizer,
    bidi_reset_sync: ResetSynchronizer,
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
        self.sys_reset_sync.clock.next = self.sys_clock.val();
        self.sys_reset_sync.reset_in.next = self.reset.val();
        self.sys_reset.next = self.sys_reset_sync.reset_out.val();
        self.bidi_reset_sync.clock.next = self.bidi_clock.val();
        self.bidi_reset_sync.reset_in.next = self.reset.val();
        self.bidi_reset.next = self.bidi_reset_sync.reset_out.val();
        clock_reset!(self, bidi_clock, bidi_reset, bidi_master);
        FIFOWriteController::<Bits<8>>::join(
            &mut self.bidi_master.data_from_bus,
            &mut self.bus_to_controller.narrow_bus,
        );
        self.bus_to_controller.narrow_clock.next = self.bidi_clock.val();
        self.bus_to_controller.narrow_reset.next = self.bidi_reset.val();
        self.bus_to_controller.wide_clock.next = self.sys_clock.val();
        self.bus_to_controller.wide_reset.next = self.sys_reset.val();
        FIFOReadController::<Bits<8>>::join(
            &mut self.bidi_master.data_to_bus,
            &mut self.controller_to_bus.narrow_bus,
        );
        self.controller_to_bus.narrow_clock.next = self.bidi_clock.val();
        self.controller_to_bus.narrow_reset.next = self.bidi_reset.val();
        self.controller_to_bus.wide_clock.next = self.sys_clock.val();
        self.controller_to_bus.wide_reset.next = self.sys_reset.val();
        FIFOReadController::<Bits<16>>::join(
            &mut self.controller.from_cpu,
            &mut self.bus_to_controller.wide_bus,
        );
        FIFOWriteController::<Bits<16>>::join(
            &mut self.controller.to_cpu,
            &mut self.controller_to_bus.wide_bus,
        );
        clock_reset!(self, sys_clock, sys_reset, controller);
        SoCBusController::<16, A>::link(&mut self.bus, &mut self.controller.bus);
    }
}

#[test]
fn test_host_synthesizes() {
    let mut uut = Host::<8>::default();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    std::fs::write("host.v", &vlog).unwrap();
    yosys_validate("host", &vlog).unwrap();
}
