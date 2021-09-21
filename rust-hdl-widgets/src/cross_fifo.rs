use crate::async_fifo::AsynchronousFIFO;
use crate::fifo_expander_n::WordOrder;
use crate::prelude::{FIFOExpanderN, FIFOReducerN};
use crate::sync_fifo::SynchronousFIFO;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::TopWrap;

#[derive(LogicBlock)]
pub struct CrossWidenFIFO<
    const DN: usize,   // Narrow width
    const NN: usize,   // Number of bits on the narrow side address
    const NNP1: usize, // NN + 1
    const DW: usize,   // Wide width
    const WN: usize,   // Number of bits on the wide side address
    const WNP1: usize, // WN + 1
> {
    // Write interface
    pub data_in: Signal<In, Bits<DN>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub write_clock: Signal<In, Clock>,
    // Read interface
    pub data_out: Signal<Out, Bits<DW>>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
    pub read_clock: Signal<In, Clock>,
    // Input FIFO
    pub in_fifo: AsynchronousFIFO<Bits<DN>, NN, NNP1, 1>,
    // Output FIFO
    pub out_fifo: SynchronousFIFO<Bits<DW>, WN, WNP1, 1>,
    // Expander
    pub xpand: FIFOExpanderN<DN, DW>,
}

impl<
        const DN: usize,
        const NN: usize,
        const NNP1: usize,
        const DW: usize,
        const WN: usize,
        const WNP1: usize,
    > CrossWidenFIFO<DN, NN, NNP1, DW, WN, WNP1>
{
    pub fn new(order: WordOrder) -> Self {
        Self {
            data_in: Default::default(),
            write: Default::default(),
            full: Default::default(),
            write_clock: Default::default(),
            data_out: Default::default(),
            read: Default::default(),
            empty: Default::default(),
            read_clock: Default::default(),
            in_fifo: Default::default(),
            out_fifo: Default::default(),
            xpand: FIFOExpanderN::new(order),
        }
    }
}

impl<
        const DN: usize,
        const NN: usize,
        const NNP1: usize,
        const DW: usize,
        const WN: usize,
        const WNP1: usize,
    > Default for CrossWidenFIFO<DN, NN, NNP1, DW, WN, WNP1>
{
    fn default() -> Self {
        Self::new(WordOrder::MostSignificantFirst)
    }
}

impl<
        const DN: usize,
        const NN: usize,
        const NNP1: usize,
        const DW: usize,
        const WN: usize,
        const WNP1: usize,
    > Logic for CrossWidenFIFO<DN, NN, NNP1, DW, WN, WNP1>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the write side of the input fifo to the write interface
        self.in_fifo.data_in.next = self.data_in.val();
        self.in_fifo.write.next = self.write.val();
        self.full.next = self.in_fifo.full.val();
        self.in_fifo.write_clock.next = self.write_clock.val();
        // Connect the read side of the input fifo to the expander
        self.xpand.data_in.next = self.in_fifo.data_out.val();
        self.xpand.empty.next = self.in_fifo.empty.val();
        self.in_fifo.read.next = self.xpand.read.val();
        self.in_fifo.read_clock.next = self.read_clock.val();
        self.xpand.clock.next = self.read_clock.val();
        // Connect the read side of the output fifo to the read interface
        self.data_out.next = self.out_fifo.data_out.val();
        self.empty.next = self.out_fifo.empty.val();
        self.out_fifo.read.next = self.read.val();
        self.out_fifo.clock.next = self.read_clock.val();
        // Connect the write side of the output fifo to the expander
        self.out_fifo.data_in.next = self.xpand.data_out.val();
        self.xpand.full.next = self.out_fifo.full.val();
        self.out_fifo.write.next = self.xpand.write.val();
    }
}

#[test]
fn cross_widen_fifo_is_synthesizable() {
    let mut dev = TopWrap::new(CrossWidenFIFO::<16, 8, 9, 128, 5, 6>::new(
        WordOrder::MostSignificantFirst,
    ));
    dev.uut.data_in.connect();
    dev.uut.write.connect();
    dev.uut.write_clock.connect();
    dev.uut.read.connect();
    dev.uut.read_clock.connect();
    dev.connect_all();
    rust_hdl_synth::yosys_validate("cross_wide", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev))
}

#[derive(LogicBlock)]
pub struct CrossNarrowFIFO<
    const DW: usize,
    const WN: usize,
    const WNP1: usize,
    const DN: usize,
    const NN: usize,
    const NNP1: usize,
> {
    // Write interface
    pub data_in: Signal<In, Bits<DW>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub write_clock: Signal<In, Clock>,
    // Read interface
    pub data_out: Signal<Out, Bits<DN>>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
    pub read_clock: Signal<In, Clock>,
    // Input FIFO
    pub in_fifo: AsynchronousFIFO<Bits<DW>, WN, WNP1, 1>,
    // Output FIFO
    pub out_fifo: SynchronousFIFO<Bits<DN>, NN, NNP1, 1>,
    // Reducer
    pub reducer: FIFOReducerN<DW, DN>,
}

impl<
        const DW: usize,
        const WN: usize,
        const WNP1: usize,
        const DN: usize,
        const NN: usize,
        const NNP1: usize,
    > CrossNarrowFIFO<DW, WN, WNP1, DN, NN, NNP1>
{
    pub fn new(order: WordOrder) -> Self {
        Self {
            data_in: Default::default(),
            write: Default::default(),
            full: Default::default(),
            write_clock: Default::default(),
            data_out: Default::default(),
            read: Default::default(),
            empty: Default::default(),
            read_clock: Default::default(),
            in_fifo: Default::default(),
            out_fifo: Default::default(),
            reducer: FIFOReducerN::new(order),
        }
    }
}

impl<
        const DW: usize,
        const WN: usize,
        const WNP1: usize,
        const DN: usize,
        const NN: usize,
        const NNP1: usize,
    > Logic for CrossNarrowFIFO<DW, WN, WNP1, DN, NN, NNP1>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Connect the write side of the input fifo to the write interface
        self.in_fifo.data_in.next = self.data_in.val();
        self.in_fifo.write.next = self.write.val();
        self.full.next = self.in_fifo.full.val();
        self.in_fifo.write_clock.next = self.write_clock.val();
        // Connect the read side of the input fifo to the reducer
        self.reducer.data_in.next = self.in_fifo.data_out.val();
        self.reducer.empty.next = self.in_fifo.empty.val();
        self.in_fifo.read.next = self.reducer.read.val();
        self.in_fifo.read_clock.next = self.read_clock.val();
        self.reducer.clock.next = self.read_clock.val();
        // Connect the read side of the output fifo to the read interface
        self.data_out.next = self.out_fifo.data_out.val();
        self.empty.next = self.out_fifo.empty.val();
        self.out_fifo.read.next = self.read.val();
        self.out_fifo.clock.next = self.read_clock.val();
        // Connect the write side of the output fifo to the reducer
        self.out_fifo.data_in.next = self.reducer.data_out.val();
        self.reducer.full.next = self.out_fifo.full.val();
        self.out_fifo.write.next = self.reducer.write.val();
    }
}

impl<
        const DW: usize,
        const WN: usize,
        const WNP1: usize,
        const DN: usize,
        const NN: usize,
        const NNP1: usize,
    > Default for CrossNarrowFIFO<DW, WN, WNP1, DN, NN, NNP1>
{
    fn default() -> Self {
        Self::new(WordOrder::MostSignificantFirst)
    }
}

#[test]
fn cross_narrow_fifo_is_synthesizable() {
    let mut dev = TopWrap::new(CrossNarrowFIFO::<128, 5, 6, 16, 8, 9>::new(
        WordOrder::MostSignificantFirst,
    ));
    dev.uut.data_in.connect();
    dev.uut.write.connect();
    dev.uut.write_clock.connect();
    dev.uut.read.connect();
    dev.uut.read_clock.connect();
    dev.connect_all();
    rust_hdl_synth::yosys_validate("cross_narrow", &generate_verilog(&dev)).unwrap();
    println!("{}", generate_verilog(&dev))
}

#[macro_export]
macro_rules! declare_expanding_fifo {
    ($name: ident, $narrow_bits: expr, $narrow_count: expr, $wide_bits: expr, $wide_count: expr) => {
        pub type $name = CrossWidenFIFO<
            $narrow_bits,
            { clog2($narrow_count) },
            { clog2($narrow_count) + 1 },
            $wide_bits,
            { clog2($wide_count) },
            { clog2($wide_count) + 1 },
        >;
    };
}

#[macro_export]
macro_rules! declare_narrowing_fifo {
    ($name: ident, $wide_bits: expr, $wide_count: expr, $narrow_bits: expr, $narrow_count: expr) => {
        pub type $name = CrossNarrowFIFO<
            $wide_bits,
            { clog2($wide_count) },
            { clog2($wide_count) + 1 },
            $narrow_bits,
            { clog2($narrow_count) },
            { clog2($narrow_count) + 1 },
        >;
    };
}
