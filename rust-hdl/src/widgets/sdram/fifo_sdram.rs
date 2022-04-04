use crate::core::prelude::*;
use crate::widgets::prelude::{
    AsynchronousFIFO, MemoryTimings, OutputBuffer, SDRAMBaseController, DFF,
};
use crate::widgets::sdram::SDRAMDriver;

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
enum State {
    Idle,
    Busy,
}

#[derive(LogicBlock)]
pub struct SDRAMFIFOController<
    const R: usize, // Number of rows in the SDRAM
    const C: usize, // Number of columns in the SDRAM
    const L: usize, // Line size (multiple of the SDRAM interface width) - rem(C, L) = 0
    const D: usize, // Number of bits in the SDRAM interface width
    const A: usize, // Number of address bits in the SDRAM (should be C + R + B)
> {
    pub clock: Signal<In, Clock>,
    pub sdram: SDRAMDriver<D>,
    pub ram_clock: Signal<In, Clock>,
    // FIFO interface
    pub data_in: Signal<In, Bits<L>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub data_out: Signal<Out, Bits<L>>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
    pub overflow: Signal<Out, Bit>,
    pub underflow: Signal<Out, Bit>,
    pub status: Signal<Out, Bits<8>>,
    core: SDRAMBaseController<R, C, L, D>,
    fp: AsynchronousFIFO<Bits<L>, 6, 7, 1>,
    bp: AsynchronousFIFO<Bits<L>, 6, 7, 1>,
    will_write: Signal<Local, Bit>,
    will_read: Signal<Local, Bit>,
    read_pointer: DFF<Bits<A>>,
    write_pointer: DFF<Bits<A>>,
    dram_is_empty: Signal<Local, Bit>,
    dram_is_full: Signal<Local, Bit>,
    state: DFF<State>,
    line_to_word_ratio: Constant<Bits<A>>,
    fill: DFF<Bits<A>>,
    status_reg: DFF<Bits<8>>,
}

impl<const R: usize, const C: usize, const L: usize, const D: usize, const A: usize>
    SDRAMFIFOController<R, C, L, D, A>
{
    pub fn new(cas_delay: u32, timings: MemoryTimings, buffer: OutputBuffer) -> Self {
        assert_eq!((1 << C) % (L / D), 0);
        assert_eq!(L % D, 0);
        assert_eq!(A, C + R + 2);
        Self {
            clock: Default::default(),
            sdram: Default::default(),
            ram_clock: Default::default(),
            data_in: Default::default(),
            write: Default::default(),
            full: Default::default(),
            data_out: Default::default(),
            read: Default::default(),
            empty: Default::default(),
            overflow: Default::default(),
            underflow: Default::default(),
            status: Default::default(),
            core: SDRAMBaseController::new(cas_delay, timings, buffer),
            fp: Default::default(),
            bp: Default::default(),
            will_write: Default::default(),
            will_read: Default::default(),
            read_pointer: Default::default(),
            write_pointer: Default::default(),
            dram_is_empty: Default::default(),
            dram_is_full: Default::default(),
            state: Default::default(),
            line_to_word_ratio: Constant::new((L / D).into()),
            fill: Default::default(),
            status_reg: Default::default(),
        }
    }
}

impl<const R: usize, const C: usize, const L: usize, const D: usize, const A: usize> Logic
    for SDRAMFIFOController<R, C, L, D, A>
{
    #[hdl_gen]
    fn update(&mut self) {
        self.core.clock.next = self.ram_clock.val();
        SDRAMDriver::<D>::link(&mut self.sdram, &mut self.core.sdram);
        self.read_pointer.clk.next = self.ram_clock.val();
        self.read_pointer.d.next = self.read_pointer.q.val();
        self.write_pointer.clk.next = self.ram_clock.val();
        self.write_pointer.d.next = self.write_pointer.q.val();
        // The FP write clock is external, but the read clock is the DRAM clock
        self.fp.write_clock.next = self.clock.val();
        self.fp.read_clock.next = self.ram_clock.val();
        // The BP write clock is DRAM, the read clock is external
        self.bp.write_clock.next = self.ram_clock.val();
        self.bp.read_clock.next = self.clock.val();
        self.state.clk.next = self.ram_clock.val();
        self.state.d.next = self.state.q.val();
        // Connect the write interface to the FP fifo
        self.fp.data_in.next = self.data_in.val();
        self.fp.write.next = self.write.val();
        self.full.next = self.fp.full.val();
        self.overflow.next = self.fp.overflow.val();
        // Connect the read interface to the BP fifo
        self.data_out.next = self.bp.data_out.val();
        self.bp.read.next = self.read.val();
        self.empty.next = self.bp.empty.val();
        self.underflow.next = self.bp.underflow.val();
        // Connect the read interface of the FP fifo to the DRAM controller
        self.core.data_in.next = self.fp.data_out.val();
        self.fp.read.next = false;
        // Connect the write interface of the DRAM controller to the BP fifo
        self.bp.data_in.next = self.core.data_out.val();
        self.bp.write.next = self.core.data_valid.val();
        // That takes care of the outside facing part of the fifo...
        //  Now the internals.
        self.dram_is_empty.next = self.read_pointer.q.val() == self.write_pointer.q.val();
        self.dram_is_full.next = (self.write_pointer.q.val() + self.line_to_word_ratio.val())
            == self.read_pointer.q.val();
        self.will_write.next = !self.dram_is_full.val() & !self.fp.empty.val();
        self.will_read.next = !self.dram_is_empty.val() & !self.bp.full.val();
        self.core.cmd_address.next = 0_usize.into();
        self.core.write_not_read.next = false;
        self.core.cmd_strobe.next = false;
        self.fill.clk.next = self.ram_clock.val();
        self.fill.d.next = self.fill.q.val();
        match self.state.q.val() {
            State::Idle => {
                if !self.core.busy.val() {
                    if self.will_read.val() {
                        self.state.d.next = State::Busy;
                        self.core.cmd_address.next = bit_cast::<32, A>(self.read_pointer.q.val());
                        self.core.write_not_read.next = false;
                        self.core.cmd_strobe.next = true;
                        self.read_pointer.d.next =
                            self.read_pointer.q.val() + self.line_to_word_ratio.val();
                        self.fill.d.next = self.fill.q.val() - 1_usize;
                    } else if self.will_write.val() {
                        self.state.d.next = State::Busy;
                        self.core.cmd_address.next = bit_cast::<32, A>(self.write_pointer.q.val());
                        self.core.write_not_read.next = true;
                        self.core.cmd_strobe.next = true;
                        self.fp.read.next = true;
                        self.write_pointer.d.next =
                            self.write_pointer.q.val() + self.line_to_word_ratio.val();
                        self.fill.d.next = self.fill.q.val() + 1_usize;
                    }
                }
            }
            State::Busy => {
                if !self.core.busy.val() {
                    self.state.d.next = State::Idle;
                }
            }
        }
        self.status_reg.clk.next = self.clock.val();
        self.status.next = self.status_reg.q.val();
        self.status_reg.d.next = 0_usize.into();
        // We have 512Mbits of memory.
        // Each write is 128bits of data
        // So the max fill is 4M of data
        // To display this on an 8 bit display, we
        // use a chunk size of 2^19.
        //
        //524288   1048576   1572864   2097152   2621440   3145728   3670016   4194304
        if self.fill.q.val() > bits::<A>(838860) {
            self.status_reg.d.next = self.status_reg.d.val() | 1_usize;
        }
        if self.fill.q.val() > bits::<A>(1677721) {
            self.status_reg.d.next = self.status_reg.d.val() | 2_usize;
        }
        if self.fill.q.val() > bits::<A>(2516582) {
            self.status_reg.d.next = self.status_reg.d.val() | 4_usize;
        }
        if self.fill.q.val() > bits::<A>(3355443) {
            self.status_reg.d.next = self.status_reg.d.val() | 8_usize;
        }
        if self.fill.q.val() > bits::<A>(4190000) {
            self.status_reg.d.next = self.status_reg.d.val() | 16_usize;
        }
        if self.dram_is_empty.val() {
            self.status_reg.d.next = self.status_reg.d.val() | 32_usize;
        }
        if self.fp.empty.val() {
            self.status_reg.d.next = self.status_reg.d.val() | 64_usize;
        }
        if self.bp.full.val() {
            self.status_reg.d.next = self.status_reg.d.val() | 128_usize;
        }
    }
}
