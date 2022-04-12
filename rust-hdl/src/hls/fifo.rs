use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadResponder, FIFOWriteResponder};
use crate::widgets::prelude::*;

#[derive(LogicBlock, Default)]
pub struct SyncFIFO<T: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    pub bus_write: FIFOWriteResponder<T>,
    pub bus_read: FIFOReadResponder<T>,
    pub clock: Signal<In, Clock>,
    pub reset: Signal<In, Reset>,
    fifo: SynchronousFIFO<T, N, NP1, BLOCK_SIZE>,
}

impl<T: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Logic
    for SyncFIFO<T, N, NP1, BLOCK_SIZE>
{
    #[hdl_gen]
    fn update(&mut self) {
        clock_reset!(self, clock, reset, fifo);
        // Connect up the write side of the FIFO
        self.fifo.data_in.next = self.bus_write.data.val();
        self.fifo.write.next = self.bus_write.write.val();
        self.bus_write.full.next = self.fifo.full.val();
        self.bus_write.almost_full.next = self.fifo.almost_full.val();
        // Connect up the read side of the FIFO
        self.bus_read.data.next = self.fifo.data_out.val();
        self.bus_read.empty.next = self.fifo.empty.val();
        self.bus_read.almost_empty.next = self.fifo.almost_empty.val();
        self.fifo.read.next = self.bus_read.read.val();
    }
}

#[derive(LogicBlock, Default)]
pub struct AsyncFIFO<T: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    pub bus_write: FIFOWriteResponder<T>,
    pub write_clock: Signal<In, Clock>,
    pub write_reset: Signal<In, Reset>,
    pub bus_read: FIFOReadResponder<T>,
    pub read_clock: Signal<In, Clock>,
    pub read_reset: Signal<In, Reset>,
    fifo: AsynchronousFIFO<T, N, NP1, BLOCK_SIZE>,
}

impl<T: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Logic
    for AsyncFIFO<T, N, NP1, BLOCK_SIZE>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect up the write side of the FIFO
        self.fifo.data_in.next = self.bus_write.data.val();
        self.fifo.write.next = self.bus_write.write.val();
        self.fifo.write_clock.next = self.write_clock.val();
        self.fifo.write_reset.next = self.write_reset.val();
        self.bus_write.full.next = self.fifo.full.val();
        self.bus_write.almost_full.next = self.fifo.almost_full.val();
        // Connect up the read side of the FIFO
        self.bus_read.data.next = self.fifo.data_out.val();
        self.bus_read.empty.next = self.fifo.empty.val();
        self.bus_read.almost_empty.next = self.fifo.almost_empty.val();
        self.fifo.read.next = self.bus_read.read.val();
        self.fifo.read_clock.next = self.read_clock.val();
        self.fifo.read_reset.next = self.read_reset.val();
    }
}

#[test]
fn test_hsl_fifo_synthesizes() {
    let mut uut = AsyncFIFO::<Bits<8>, 6, 7, 1>::default();
    uut.write_clock.connect();
    uut.write_reset.connect();
    uut.bus_write.write.connect();
    uut.bus_write.data.connect();
    uut.read_clock.connect();
    uut.read_reset.connect();
    uut.bus_read.read.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hsl_fifo", &vlog).unwrap();
}

#[test]
fn test_hsl_sync_fifo_synthesizes() {
    let mut uut = SyncFIFO::<Bits<8>, 6, 7, 1>::default();
    uut.clock.connect();
    uut.reset.connect();
    uut.bus_write.write.connect();
    uut.bus_write.data.connect();
    uut.bus_read.read.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hsl_sync_fifo", &vlog).unwrap();
}
