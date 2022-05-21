use crate::core::prelude::*;
use crate::hls::bus::{FIFOReadResponder, FIFOWriteResponder};
use crate::widgets::prelude::*;

#[derive(LogicBlock, Default)]
pub struct CrossWiden<
    const DN: usize,
    const NN: usize,
    const NNP1: usize,
    const DW: usize,
    const WN: usize,
    const WNP1: usize,
> {
    pub narrow_bus: FIFOWriteResponder<Bits<DN>>,
    pub narrow_clock: Signal<In, Clock>,
    pub wide_bus: FIFOReadResponder<Bits<DW>>,
    pub wide_clock: Signal<In, Clock>,
    widen: CrossWidenFIFO<DN, NN, NNP1, DW, WN, WNP1>,
}

impl<
        const DN: usize,
        const NN: usize,
        const NNP1: usize,
        const DW: usize,
        const WN: usize,
        const WNP1: usize,
    > CrossWiden<DN, NN, NNP1, DW, WN, WNP1>
{
    pub fn new(order: WordOrder) -> Self {
        Self {
            narrow_bus: Default::default(),
            narrow_clock: Default::default(),
            wide_bus: Default::default(),
            wide_clock: Default::default(),
            widen: CrossWidenFIFO::new(order),
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
    > Logic for CrossWiden<DN, NN, NNP1, DW, WN, WNP1>
{
    #[hdl_gen]
    fn update(&mut self) {
        // Wire up the input side
        self.widen.data_in.next = self.narrow_bus.data.val();
        self.widen.write.next = self.narrow_bus.write.val();
        self.narrow_bus.full.next = self.widen.full.val();
        self.narrow_bus.almost_full.next = self.widen.full.val();
        self.widen.write_clock.next = self.narrow_clock.val();
        // Wire up the output side
        self.wide_bus.data.next = self.widen.data_out.val();
        self.wide_bus.empty.next = self.widen.empty.val();
        self.wide_bus.almost_empty.next = self.widen.empty.val();
        self.widen.read.next = self.wide_bus.read.val();
        self.widen.read_clock.next = self.wide_clock.val();
    }
}

#[test]
fn test_hsl_cross_fifo_synthesizes() {
    let mut uut: CrossWiden<4, 5, 6, 16, 3, 4> = CrossWiden::new(WordOrder::LeastSignificantFirst);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hsl_cross_fifo", &vlog).unwrap();
}

#[derive(LogicBlock, Default)]
pub struct CrossNarrow<
    const DW: usize,
    const WN: usize,
    const WNP1: usize,
    const DN: usize,
    const NN: usize,
    const NNP1: usize,
> {
    pub wide_bus: FIFOWriteResponder<Bits<DW>>,
    pub wide_clock: Signal<In, Clock>,
    pub narrow_bus: FIFOReadResponder<Bits<DN>>,
    pub narrow_clock: Signal<In, Clock>,
    narrow: CrossNarrowFIFO<DW, WN, WNP1, DN, NN, NNP1>,
}

impl<
        const DW: usize,
        const WN: usize,
        const WNP1: usize,
        const DN: usize,
        const NN: usize,
        const NNP1: usize,
    > CrossNarrow<DW, WN, WNP1, DN, NN, NNP1>
{
    pub fn new(order: WordOrder) -> Self {
        Self {
            wide_bus: Default::default(),
            wide_clock: Default::default(),
            narrow_bus: Default::default(),
            narrow_clock: Default::default(),
            narrow: CrossNarrowFIFO::new(order),
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
    > Logic for CrossNarrow<DW, WN, WNP1, DN, NN, NNP1>
{
    #[hdl_gen]
    fn update(&mut self) {
        self.narrow.data_in.next = self.wide_bus.data.val();
        self.narrow.write.next = self.wide_bus.write.val();
        self.wide_bus.full.next = self.narrow.full.val();
        self.wide_bus.almost_full.next = self.narrow.full.val();
        self.narrow.write_clock.next = self.wide_clock.val();
        self.narrow_bus.data.next = self.narrow.data_out.val();
        self.narrow_bus.empty.next = self.narrow.empty.val();
        self.narrow_bus.almost_empty.next = self.narrow.empty.val();
        self.narrow.read.next = self.narrow_bus.read.val();
        self.narrow.read_clock.next = self.narrow_clock.val();
    }
}

#[test]
fn test_hsl_cross_narrow_synthesizes() {
    let mut uut: CrossNarrow<16, 3, 4, 4, 5, 6> =
        CrossNarrow::new(WordOrder::LeastSignificantFirst);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hsl_cross_narrow_fifo", &vlog).unwrap();
}
