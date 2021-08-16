use rust_hdl_core::prelude::*;

use crate::fifo_components::{FIFOReadLogic, FIFOWriteLogic};
use crate::fifo_if::{FIFOReadIF, FIFOWriteIF};
use crate::ram::RAM;

macro_rules! declare_fifo {
    ($name: ident, $kind: ty, $count: expr, $block: expr) => {
            pub type $name = SynchronousFIFO<$kind, {clog2($count)}, {clog2($count)+1}, $block>;
    }
}


#[derive(LogicBlock, Default)]
pub struct SynchronousFIFO<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    pub clock: Signal<In, Clock>,
    pub read_if: FIFOReadIF<D>,
    pub write_if: FIFOWriteIF<D>,
    // Internal RAM
    ram: RAM<D, N>,
    // Read logic
    read: FIFOReadLogic<D, N, NP1, BLOCK_SIZE>,
    // write logic
    write: FIFOWriteLogic<D, N, NP1, BLOCK_SIZE>,
}

// Ported from fifo.luc in AlchitryLabs under MIT license
// Modified to use an extra bit for the read/write address
// pointers per:
// http://www.sunburst-design.com/papers/CummingsSNUG2002SJ_FIFO1.pdf
// That modification allows you to fill the FIFO all the way
// since you have 2 different conditions for full and empty.
// The design was also split into sub-circuits to allow reuse,
// following the suggestion in the Cummings paper.
// Note that we could skip the read_if, and write_if headers
// and connect directly to the read/write circuitry from outside
// but that this makes the FIFO less pleasant to use, since
// you then need fifo.write.sig.write, instead of
// fifo.write_if.write
impl<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Logic
    for SynchronousFIFO<D, N, NP1, BLOCK_SIZE>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect up the read interface
        self.read.clock.next = self.clock.val();
        self.read.sig.read.next = self.read_if.read.val();
        self.read_if.empty.next = self.read.sig.empty.val();
        self.read_if.almost_empty.next = self.read.sig.almost_empty.val();
        self.read_if.data_out.next = self.read.sig.data_out.val();
        self.read_if.underflow.next = self.read.sig.underflow.val();
        // Connect up the write interface
        self.write.clock.next = self.clock.val();
        self.write_if.overflow.next = self.write.sig.overflow.val();
        self.write_if.almost_full.next = self.write.sig.almost_full.val();
        self.write_if.full.next = self.write.sig.full.val();
        self.write.sig.write.next = self.write_if.write.val();
        self.write.sig.data_in.next = self.write_if.data_in.val();
        // Connect the RAM to the two blocks
        self.ram.write.clock.next = self.clock.val();
        self.ram.write.enable.next = self.write.ram_write.enable.val();
        self.ram.write.address.next = self.write.ram_write.address.val();
        self.ram.write.data.next = self.write.ram_write.data.val();
        self.ram.read.clock.next = self.clock.val();
        self.ram.read.address.next = self.read.ram_read.address.val();
        self.read.ram_read.data.next = self.ram.read.data.val();
        // Connect the two blocks
        self.read.write_address_delayed.next = self.write.write_address_delayed.val();
        self.write.read_address.next = self.read.read_address_out.val();
    }
}

#[test]
fn component_fifo_is_synthesizable() {
    rust_hdl_synth::top_wrap!(SynchronousFIFO<Bits<8>, 4, 5, 1>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.read_if.read.connect();
    dev.uut.write_if.write.connect();
    dev.uut.write_if.data_in.connect();
    dev.connect_all();
    rust_hdl_synth::yosys_validate("fifo", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}

#[test]
fn test_fifo_macro() {
    declare_fifo!(FIFOTest, Bits<8>, 32, 1);
    let dev = FIFOTest::default();
}