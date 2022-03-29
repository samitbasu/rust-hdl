use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadResponder, FIFOWriteResponder};
use crate::widgets::prelude::*;
use crate::widgets::sdram::SDRAMDriver;

#[derive(LogicBlock)]
pub struct SDRAMFIFO<
    const R: usize,
    const C: usize,
    const P: usize,
    const D: usize,
    const A: usize,
    const AP1: usize,
> {
    pub clock: Signal<In, Clock>,
    pub sdram: SDRAMDriver<D>,
    pub bus_write: FIFOWriteResponder<Bits<P>>,
    pub bus_read: FIFOReadResponder<Bits<P>>,
    controller: SDRAMFIFOController<R, C, P, D, A, AP1>,
}

impl<const R: usize, const C: usize, const P: usize, const D: usize, const A: usize, const AP1: usize> Logic
    for SDRAMFIFO<R, C, P, D, A, AP1>
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

impl<const R: usize, const C: usize, const P: usize, const D: usize, const A: usize, const AP1: usize> SDRAMFIFO<R, C, P, D, A, AP1> {
    pub fn new(cas_delay: u32, timings: MemoryTimings, buffer: OutputBuffer) -> SDRAMFIFO<R, C, P, D, A, AP1> {
        Self {
            clock: Default::default(),
            sdram: Default::default(),
            bus_write: Default::default(),
            bus_read: Default::default(),
            controller: SDRAMFIFOController::new(cas_delay, timings, buffer),
        }
    }
}

#[test]
fn test_sdram_fifo_synthesizes() {
    let mut uut = SDRAMFIFO::<6, 4, 64, 16, 10, 11>::new(3, MemoryTimings::fast_boot_sim(125e6), OutputBuffer::Wired);
    uut.clock.connect();
    uut.bus_read.link_connect_dest();
    uut.bus_write.link_connect_dest();
    uut.sdram.read_data.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    std::fs::write("sdram_fifo_hls.v", &vlog).unwrap();
    yosys_validate("sdram_fifo_hls", &vlog).unwrap();
}
