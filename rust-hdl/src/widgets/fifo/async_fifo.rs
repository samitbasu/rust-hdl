use crate::core::prelude::*;
use crate::widgets::fifo::fifo_logic::{FIFOReadLogic, FIFOWriteLogic};
use crate::widgets::ramrom::ram::RAM;
use crate::widgets::synchronizer::VectorSynchronizer;

#[macro_export]
macro_rules! declare_async_fifo {
    ($name: ident, $kind: ty, $count: expr, $block: expr) => {
        pub type $name = AsynchronousFIFO<$kind, { clog2($count) }, { clog2($count) + 1 }, $block>;
    };
}

#[derive(LogicBlock, Default)]
pub struct AsynchronousFIFO<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> {
    // Read interface
    pub read: Signal<In, Bit>,
    pub data_out: Signal<Out, D>,
    pub empty: Signal<Out, Bit>,
    pub almost_empty: Signal<Out, Bit>,
    pub underflow: Signal<Out, Bit>,
    pub read_clock: Signal<In, Clock>,
    pub read_fill: Signal<Out, Bits<NP1>>,
    // Write interface
    pub write: Signal<In, Bit>,
    pub data_in: Signal<In, D>,
    pub full: Signal<Out, Bit>,
    pub almost_full: Signal<Out, Bit>,
    pub overflow: Signal<Out, Bit>,
    pub write_clock: Signal<In, Clock>,
    pub write_fill: Signal<Out, Bits<NP1>>,
    // Internal RAM
    ram: RAM<D, N>,
    // Read Logic
    read_logic: FIFOReadLogic<D, N, NP1, BLOCK_SIZE>,
    // write logic
    write_logic: FIFOWriteLogic<D, N, NP1, BLOCK_SIZE>,
    // Synchronize the write pointer to the read side
    write_to_read: VectorSynchronizer<Bits<NP1>>,
    // Synchronize the read pointer to the write side
    read_to_write: VectorSynchronizer<Bits<NP1>>,
}

impl<D: Synth, const N: usize, const NP1: usize, const BLOCK_SIZE: u32> Logic
    for AsynchronousFIFO<D, N, NP1, BLOCK_SIZE>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect up the read interface
        self.read_logic.clock.next = self.read_clock.val();
        self.read_logic.read.next = self.read.val();
        self.empty.next = self.read_logic.empty.val();
        self.almost_empty.next = self.read_logic.almost_empty.val();
        self.data_out.next = self.read_logic.data_out.val();
        self.underflow.next = self.read_logic.underflow.val();
        // Connect up the write interface
        self.write_logic.clock.next = self.write_clock.val();
        self.overflow.next = self.write_logic.overflow.val();
        self.almost_full.next = self.write_logic.almost_full.val();
        self.full.next = self.write_logic.full.val();
        self.write_logic.write.next = self.write.val();
        self.write_logic.data_in.next = self.data_in.val();
        // Connect the RAM to the two blocks
        self.ram.write_clock.next = self.write_logic.ram_write_clock.val();
        self.ram.write_enable.next = self.write_logic.ram_write_enable.val();
        self.ram.write_address.next = self.write_logic.ram_write_address.val();
        self.ram.write_data.next = self.write_logic.ram_write_data.val();
        self.ram.read_clock.next = self.read_logic.ram_read_clock.val();
        self.ram.read_address.next = self.read_logic.ram_read_address.val();
        self.read_logic.ram_read_data.next = self.ram.read_data.val();
        // Connect the read block --> write block via a synchronizer
        self.read_to_write.clock_in.next = self.read_clock.val();
        self.read_to_write.clock_out.next = self.write_clock.val();
        self.read_to_write.sig_in.next = self.read_logic.read_address_out.val();
        self.write_logic.read_address.next = self.read_to_write.sig_out.val();
        self.read_to_write.send.next = !self.read_to_write.busy.val();
        // Connect the write block --> read block via a synchronizer
        self.write_to_read.clock_in.next = self.write_clock.val();
        self.write_to_read.clock_out.next = self.read_clock.val();
        self.write_to_read.sig_in.next = self.write_logic.write_address_delayed.val();
        self.read_logic.write_address_delayed.next = self.write_to_read.sig_out.val();
        self.write_to_read.send.next = !self.write_to_read.busy.val();
        // Provide the fill level estimates
        self.write_fill.next = self.write_logic.fill_level.val();
        self.read_fill.next = self.read_logic.fill_level.val();
    }
}

#[test]
fn component_async_fifo_is_synthesizable() {
    declare_async_fifo!(TFifo, Bits<8>, 16, 1);
    let mut dev: TFifo = Default::default();
    dev.connect_all();
    yosys_validate("async_fifo", &generate_verilog(&dev)).unwrap();
}
