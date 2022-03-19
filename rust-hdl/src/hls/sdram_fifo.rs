use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadResponder, FIFOWriteResponder};
use crate::widgets::prelude::*;
use crate::widgets::sdram::SDRAMDriver;

#[derive(LogicBlock)]
pub struct SDRAMFIFO<const R: usize, const C: usize, const P: usize, const D: usize> {
    pub clock: Signal<In, Clock>,
    pub sdram: SDRAMDriver<D>,
    pub bus_write: FIFOWriteResponder<Bits<D>>,
    pub bus_read: FIFOReadResponder<Bits<D>>,
    controller: SDRAMFIFOController<R, C, P, D>,
}

impl<const R: usize, const C: usize, const P: usize, const D: usize> Logic
    for SDRAMFIFO<R, C, P, D>
{
    #[hdl_gen]
    fn update(&mut self) {
        self.controller.data_in.next = self.bus_write.data.val();
        self.controller.write.next = self.bus_write.write.val();
        self.bus_write.full.next = self.controller.full.val();
        self.bus_write.almost_full.next = self.controller.full.val();
        self.bus_read.data.next = self.controller.data_out.val();
        self.bus_read.empty.next = self.controller.empty.val();
        self.bus_read.almost_empty.next = self.controller.empty.val();
        self.controller.read.next = self.bus_read.read.val();
        self.controller.clock.next = self.clock.val();
        SDRAMDriver::<D>::link(&mut self.sdram, &mut self.controller.sdram);
    }
}

impl<const R: usize, const C: usize, const P: usize, const D: usize> SDRAMFIFO<R, C, P, D> {
    pub fn new(cas_delay: u32, timings: MemoryTimings) -> SDRAMFIFO<R, C, P, D> {
        Self {
            clock: Default::default(),
            sdram: Default::default(),
            bus_write: Default::default(),
            bus_read: Default::default(),
            controller: SDRAMFIFOController::new(cas_delay, timings),
        }
    }
}

#[test]
fn test_sdram_fifo_synthesizes() {
    let mut uut = SDRAMFIFO::<5, 5, 12, 16>::new(3, MemoryTimings::fast_boot_sim(125e6));
    uut.clock.connect();
    uut.bus_read.link_connect_dest();
    uut.bus_write.link_connect_dest();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("sdram_fifo_hls", &vlog).unwrap();
}
