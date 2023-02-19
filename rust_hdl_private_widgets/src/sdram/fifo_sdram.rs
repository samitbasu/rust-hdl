use crate::prelude::{AsynchronousFIFO, MemoryTimings, OutputBuffer, SDRAMBurstController};
use crate::sdram::SDRAMDriver;
use rust_hdl_private_core::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
enum State {
    Idle,
    Read,
    Write,
    Busy,
}

#[derive(LogicBlock)]
pub struct SDRAMFIFOController<
    const R: usize, // Number of rows in the SDRAM
    const C: usize, // Number of columns in the SDRAM
    const L: u32,   // Line size (multiple of the SDRAM interface width) - rem(2^C, L) = 0
    const D: usize, // Number of bits in the SDRAM interface width
    const A: usize, // Number of address bits in the SDRAM (should be C + R + B)
> {
    pub clock: Signal<In, Clock>,
    pub sdram: SDRAMDriver<D>,
    pub ram_clock: Signal<In, Clock>,
    // FIFO interface
    pub data_in: Signal<In, Bits<D>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub data_out: Signal<Out, Bits<D>>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
    pub overflow: Signal<Out, Bit>,
    pub underflow: Signal<Out, Bit>,
    pub status: Signal<Out, Bits<8>>,
    controller: SDRAMBurstController<R, C, L, D>,
    fp: AsynchronousFIFO<Bits<D>, 5, 6, L>,
    bp: AsynchronousFIFO<Bits<D>, 5, 6, L>,
    can_write: DFF<Bit>,
    can_read: DFF<Bit>,
    read_pointer: DFF<Bits<A>>,
    write_pointer: DFF<Bits<A>>,
    dram_is_empty: DFF<Bit>,
    dram_is_full: DFF<Bit>,
    state: DFF<State>,
    line_to_word_ratio: Constant<Bits<A>>,
    fill: DFF<Bits<A>>,
    fill_0: Constant<Bits<A>>,
    fill_1: Constant<Bits<A>>,
    fill_2: Constant<Bits<A>>,
    fill_3: Constant<Bits<A>>,
}

impl<const R: usize, const C: usize, const L: u32, const D: usize, const A: usize>
    SDRAMFIFOController<R, C, L, D, A>
{
    pub fn new(cas_delay: u32, timings: MemoryTimings, buffer: OutputBuffer) -> Self {
        assert_eq!((1 << C) % L, 0);
        assert_eq!(A, C + R + 2);
        assert!(L < 32);
        let banks = 4; // Assume 4 banks
                       // Number of total bits
        let bit_count = R * C * D * banks;
        // The number of bits in each entry
        let entry_bits = L as usize * D;
        let max_entries = bit_count / entry_bits;
        // So the number of elements we want to mark with each LED is N/5
        let entries_per_led = max_entries / 5;
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
            controller: SDRAMBurstController::new(cas_delay, timings, buffer),
            fp: Default::default(),
            bp: Default::default(),
            can_write: Default::default(),
            can_read: Default::default(),
            read_pointer: Default::default(),
            write_pointer: Default::default(),
            dram_is_empty: Default::default(),
            dram_is_full: Default::default(),
            state: Default::default(),
            line_to_word_ratio: Constant::new(L.to_bits()),
            fill: Default::default(),
            fill_0: Constant::new((entries_per_led).to_bits()),
            fill_1: Constant::new((entries_per_led * 2).to_bits()),
            fill_2: Constant::new((entries_per_led * 3).to_bits()),
            fill_3: Constant::new((entries_per_led * 4).to_bits()),
        }
    }
}

impl<const R: usize, const C: usize, const L: u32, const D: usize, const A: usize> Logic
    for SDRAMFIFOController<R, C, L, D, A>
{
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, ram_clock, controller);
        SDRAMDriver::<D>::link(&mut self.sdram, &mut self.controller.sdram);
        dff_setup!(
            self,
            ram_clock,
            read_pointer,
            write_pointer,
            dram_is_empty,
            dram_is_full,
            can_read,
            can_write,
            state,
            fill
        );
        // The FP write clock is external, but the read clock is the DRAM clock
        self.fp.write_clock.next = self.clock.val();
        self.fp.read_clock.next = self.ram_clock.val();
        // The BP write clock is DRAM, the read clock is external
        self.bp.write_clock.next = self.ram_clock.val();
        self.bp.read_clock.next = self.clock.val();
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
        // The almost empty/full signals in the async fifos are in the
        // DRAM clock domain, so we need to synchronize them back
        // Connect the read interface of the FP fifo to the DRAM controller
        self.controller.data_in.next = self.fp.data_out.val();
        self.fp.read.next = self.controller.data_strobe.val();
        // Connect the write interface of the DRAM controller to the BP fifo
        self.bp.data_in.next = self.controller.data_out.val();
        self.bp.write.next = self.controller.data_valid.val();
        // That takes care of the outside facing part of the fifo...
        //  Now the internals.
        self.dram_is_empty.d.next = self.read_pointer.q.val() == self.write_pointer.q.val();
        self.dram_is_full.d.next = (self.write_pointer.q.val() + self.line_to_word_ratio.val())
            == self.read_pointer.q.val();
        self.can_write.d.next = !self.dram_is_full.q.val() & !self.fp.almost_empty.val();
        self.can_read.d.next = !self.dram_is_empty.q.val() & !self.bp.almost_full.val();
        self.controller.cmd_address.next = 0.into();
        self.controller.write_not_read.next = false;
        self.controller.cmd_strobe.next = false;
        self.fill.clock.next = self.ram_clock.val();
        self.fill.d.next = self.fill.q.val();
        match self.state.q.val() {
            State::Idle => {
                if !self.controller.busy.val() {
                    if self.can_read.q.val() {
                        self.state.d.next = State::Read;
                        self.controller.cmd_address.next =
                            bit_cast::<32, A>(self.read_pointer.q.val());
                        self.controller.write_not_read.next = false;
                        self.controller.cmd_strobe.next = true;
                    } else if self.can_write.q.val() {
                        self.state.d.next = State::Write;
                        self.controller.cmd_address.next =
                            bit_cast::<32, A>(self.write_pointer.q.val());
                        self.controller.write_not_read.next = true;
                        self.controller.cmd_strobe.next = true;
                    }
                }
            }
            State::Read => {
                self.read_pointer.d.next =
                    self.read_pointer.q.val() + self.line_to_word_ratio.val();
                self.fill.d.next = self.fill.q.val() - 1;
                self.state.d.next = State::Busy;
            }
            State::Write => {
                self.write_pointer.d.next =
                    self.write_pointer.q.val() + self.line_to_word_ratio.val();
                self.fill.d.next = self.fill.q.val() + 1;
                self.state.d.next = State::Busy;
            }
            State::Busy => {
                if !self.controller.busy.val() {
                    self.state.d.next = State::Idle;
                }
            }
            _ => {
                self.state.d.next = State::Idle;
            }
        }
        self.status.next = 0.into();
        if self.fill.q.val() > self.fill_0.val() {
            self.status.next = self.status.val() | 1;
        }
        if self.fill.q.val() > self.fill_1.val() {
            self.status.next = self.status.val() | 2;
        }
        if self.fill.q.val() > self.fill_2.val() {
            self.status.next = self.status.val() | 4;
        }
        if self.fill.q.val() > self.fill_3.val() {
            self.status.next = self.status.val() | 8;
        }
        if self.underflow.val() | self.overflow.val() {
            self.status.next = self.status.val() | 16;
        }
        if self.dram_is_empty.q.val() {
            self.status.next = self.status.val() | 32;
        }
        if self.fp.empty.val() {
            self.status.next = self.status.val() | 64;
        }
        if self.bp.full.val() {
            self.status.next = self.status.val() | 128;
        }
    }
}
