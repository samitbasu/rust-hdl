use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadResponder, FIFOWriteResponder};
use crate::widgets::prelude::*;

#[derive(LogicBlock, Default)]
pub struct HLSFIFO<const D: usize, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    pub bus_write: FIFOWriteResponder<D>,
    pub bus_read: FIFOReadResponder<D>,
    fifo: AsynchronousFIFO<Bits<D>, N, NP1, BLOCK_SIZE>,
}

impl<const D: usize, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Logic
    for HLSFIFO<D, N, NP1, BLOCK_SIZE>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect up the write side of the FIFO
        self.fifo.data_in.next = self.bus_write.data.val();
        self.fifo.write.next = self.bus_write.write.val();
        self.fifo.write_clock.next = self.bus_write.clock.val();
        self.bus_write.full.next = self.fifo.full.val();
        self.bus_write.almost_full.next = self.fifo.almost_full.val();
        // Connect up the read side of the FIFO
        self.bus_read.data.next = self.fifo.data_out.val();
        self.bus_read.empty.next = self.fifo.empty.val();
        self.bus_read.almost_empty.next = self.fifo.almost_empty.val();
        self.fifo.read.next = self.bus_read.read.val();
        self.fifo.read_clock.next = self.bus_read.clock.val();
    }
}

#[test]
fn test_hsl_fifo_synthesizes() {
    let mut uut = HLSFIFO::<8, 6, 7, 1>::default();
    uut.bus_write.clock.connect();
    uut.bus_write.write.connect();
    uut.bus_write.data.connect();
    uut.bus_read.clock.connect();
    uut.bus_read.read.connect();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hsl_fifo", &vlog).unwrap();
}
