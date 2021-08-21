use rust_hdl_core::prelude::*;

use crate::fifo_logic::{FIFOReadLogic, FIFOWriteLogic};
use crate::ram::RAM;

#[macro_export]
macro_rules! declare_sync_fifo {
    ($name: ident, $kind: ty, $count: expr, $block: expr) => {
        pub type $name = SynchronousFIFO<$kind, { clog2($count) }, { clog2($count) + 1 }, $block>;
    };
}

#[derive(LogicBlock, Default)]
pub struct SynchronousFIFO<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    pub clock: Signal<In, Clock>,
    // Read interface
    pub read: Signal<In, Bit>,
    pub data_out: Signal<Out, D>,
    pub empty: Signal<Out, Bit>,
    pub almost_empty: Signal<Out, Bit>,
    pub underflow: Signal<Out, Bit>,
    // Write interface
    pub write: Signal<In, Bit>,
    pub data_in: Signal<In, D>,
    pub full: Signal<Out, Bit>,
    pub almost_full: Signal<Out, Bit>,
    pub overflow: Signal<Out, Bit>,
    // Internal RAM
    ram: RAM<D, N>,
    // Read logic
    read_logic: FIFOReadLogic<D, N, NP1, BLOCK_SIZE>,
    // write logic
    write_logic: FIFOWriteLogic<D, N, NP1, BLOCK_SIZE>,
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
        self.read_logic.clock.next = self.clock.val();
        self.read_logic.read.next = self.read.val();
        self.empty.next = self.read_logic.empty.val();
        self.almost_empty.next = self.read_logic.almost_empty.val();
        self.data_out.next = self.read_logic.data_out.val();
        self.underflow.next = self.read_logic.underflow.val();
        // Connect up the write interface
        self.write_logic.clock.next = self.clock.val();
        self.overflow.next = self.write_logic.overflow.val();
        self.almost_full.next = self.write_logic.almost_full.val();
        self.full.next = self.write_logic.full.val();
        self.write_logic.write.next = self.write.val();
        self.write_logic.data_in.next = self.data_in.val();
        // Connect the RAM to the two blocks
        self.ram.write_clock.next = self.clock.val();
        self.ram.write_enable.next = self.write_logic.ram_write_enable.val();
        self.ram.write_address.next = self.write_logic.ram_write_address.val();
        self.ram.write_data.next = self.write_logic.ram_write_data.val();
        self.ram.read_clock.next = self.clock.val();
        self.ram.read_address.next = self.read_logic.ram_read_address.val();
        self.read_logic.ram_read_data.next = self.ram.read_data.val();
        // Connect the two blocks
        self.read_logic.write_address_delayed.next = self.write_logic.write_address_delayed.val();
        self.write_logic.read_address.next = self.read_logic.read_address_out.val();
    }
}

#[test]
fn component_fifo_is_synthesizable() {
    rust_hdl_synth::top_wrap!(SynchronousFIFO<Bits<8>, 4, 5, 1>, Wrapper);
    let mut dev: Wrapper = Default::default();
    dev.uut.clock.connect();
    dev.uut.read.connect();
    dev.uut.write.connect();
    dev.uut.data_in.connect();
    dev.connect_all();
    rust_hdl_synth::yosys_validate("fifo", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev));
}

#[test]
fn test_fifo_macro() {
    declare_sync_fifo!(FIFOTest, Bits<8>, 32, 1);
    let _dev = FIFOTest::default();
}
