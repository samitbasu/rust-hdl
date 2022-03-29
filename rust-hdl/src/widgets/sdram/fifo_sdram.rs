use crate::core::prelude::*;
use crate::sim::sdr_sdram::chip::SDRAMSimulator;
use crate::widgets::prelude::{
    MemoryTimings, OutputBuffer, SDRAMBaseController, SynchronousFIFO, DFF,
};
use crate::widgets::sdram::buffer::SDRAMOnChipBuffer;
use crate::widgets::sdram::SDRAMDriver;

#[derive(Copy, Clone, Debug, PartialEq, LogicState)]
enum State {
    Idle,
    Busy,
}

#[derive(LogicBlock)]
pub struct SDRAMFIFOController<
    const R: usize,   // Number of rows in the SDRAM
    const C: usize,   // Number of columns in the SDRAM
    const L: usize,   // Line size (multiple of the SDRAM interface width) - rem(C, L) = 0
    const D: usize,   // Number of bits in the SDRAM interface width
    const A: usize,   // Number of address bits in the SDRAM (should be C + R)
    const AP1: usize, // A + 1
> {
    pub clock: Signal<In, Clock>,
    pub sdram: SDRAMDriver<D>,
    pub write_enable: Signal<Out, Bit>,
    // FIFO interface
    pub data_in: Signal<In, Bits<L>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    pub data_out: Signal<Out, Bits<L>>,
    pub read: Signal<In, Bit>,
    pub empty: Signal<Out, Bit>,
    pub overflow: Signal<Out, Bit>,
    pub underflow: Signal<Out, Bit>,
    core: SDRAMBaseController<R, C, L, D>,
    fp: SynchronousFIFO<Bits<L>, 6, 7, 1>,
    bp: SynchronousFIFO<Bits<L>, 6, 7, 1>,
    will_write: Signal<Local, Bit>,
    will_read: Signal<Local, Bit>,
    fill_level: DFF<Bits<AP1>>,
    read_pointer: DFF<Bits<AP1>>,
    write_pointer: DFF<Bits<AP1>>,
    read_address: Signal<Local, Bits<A>>,
    write_address: Signal<Local, Bits<A>>,
    dram_is_empty: Signal<Local, Bit>,
    dram_is_full: Signal<Local, Bit>,
    state: DFF<State>,
    line_to_word_ratio: Constant<Bits<AP1>>,
}

impl<
        const R: usize,
        const C: usize,
        const L: usize,
        const D: usize,
        const A: usize,
        const AP1: usize,
    > SDRAMFIFOController<R, C, L, D, A, AP1>
{
    pub fn new(cas_delay: u32, timings: MemoryTimings, buffer: OutputBuffer) -> Self {
        assert_eq!((1 << C) % (L / D), 0);
        assert_eq!(L % D, 0);
        assert_eq!(A + 1, AP1);
        assert_eq!(A, C + R);
        Self {
            clock: Default::default(),
            sdram: Default::default(),
            write_enable: Default::default(),
            data_in: Default::default(),
            write: Default::default(),
            full: Default::default(),
            data_out: Default::default(),
            read: Default::default(),
            empty: Default::default(),
            overflow: Default::default(),
            underflow: Default::default(),
            core: SDRAMBaseController::new(cas_delay, timings, buffer),
            fp: Default::default(),
            bp: Default::default(),
            will_write: Default::default(),
            will_read: Default::default(),
            fill_level: Default::default(),
            read_pointer: Default::default(),
            write_pointer: Default::default(),
            read_address: Default::default(),
            write_address: Default::default(),
            dram_is_empty: Default::default(),
            dram_is_full: Default::default(),
            state: Default::default(),
            line_to_word_ratio: Constant::new((L / D).into()),
        }
    }
}

impl<
        const R: usize,
        const C: usize,
        const L: usize,
        const D: usize,
        const A: usize,
        const AP1: usize,
    > Logic for SDRAMFIFOController<R, C, L, D, A, AP1>
{
    #[hdl_gen]
    fn update(&mut self) {
        self.core.clock.next = self.clock.val();
        SDRAMDriver::<D>::link(&mut self.sdram, &mut self.core.sdram);
        self.write_enable.next = self.core.write_enable.val();
        self.fill_level.clk.next = self.clock.val();
        self.fill_level.d.next = self.fill_level.q.val();
        self.read_pointer.clk.next = self.clock.val();
        self.read_pointer.d.next = self.read_pointer.q.val();
        self.write_pointer.clk.next = self.clock.val();
        self.write_pointer.d.next = self.write_pointer.q.val();
        self.fp.clock.next = self.clock.val();
        self.bp.clock.next = self.clock.val();
        self.state.clk.next = self.clock.val();
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
        self.read_address.next = self.read_pointer.q.val().get_bits::<A>(0_usize);
        self.write_address.next = self.write_pointer.q.val().get_bits::<A>(0_usize);
        self.dram_is_empty.next = self.read_pointer.q.val() == self.write_pointer.q.val();
        self.dram_is_full.next =
            !self.dram_is_empty.val() & (self.read_address.val() == self.write_address.val());
        self.will_write.next = !self.dram_is_full.val() & !self.fp.empty.val();
        self.will_read.next = !self.dram_is_empty.val() & !self.bp.full.val();
        self.core.cmd_address.next = 0_usize.into();
        self.core.write_not_read.next = false;
        self.core.cmd_strobe.next = false;
        match self.state.q.val() {
            State::Idle => {
                if !self.core.busy.val() {
                    if self.will_read.val() {
                        self.state.d.next = State::Busy;
                        self.core.cmd_address.next = bit_cast::<32, A>(self.read_address.val());
                        self.core.write_not_read.next = false;
                        self.core.cmd_strobe.next = true;
                        self.read_pointer.d.next =
                            self.read_pointer.q.val() + self.line_to_word_ratio.val();
                    } else if self.will_write.val() {
                        self.state.d.next = State::Busy;
                        self.core.cmd_address.next = bit_cast::<32, A>(self.write_address.val());
                        self.core.write_not_read.next = true;
                        self.core.cmd_strobe.next = true;
                        self.fp.read.next = true;
                        self.write_pointer.d.next =
                            self.write_pointer.q.val() + self.line_to_word_ratio.val();
                    }
                }
            }
            State::Busy => {
                if !self.core.busy.val() {
                    self.state.d.next = State::Idle;
                }
            }
        }
    }
}

#[cfg(test)]
#[derive(LogicBlock)]
struct FIFOSDRAMTest {
    dram: SDRAMSimulator<6, 4, 10, 16>,
    buffer: SDRAMOnChipBuffer<16>,
    fifo: SDRAMFIFOController<6, 4, 64, 16, 10, 11>,
    clock: Signal<In, Clock>,
}

#[cfg(test)]
impl Logic for FIFOSDRAMTest {
    #[hdl_gen]
    fn update(&mut self) {
        SDRAMDriver::<16>::join(&mut self.fifo.sdram, &mut self.buffer.buf_in);
        SDRAMDriver::<16>::join(&mut self.buffer.buf_out, &mut self.dram.sdram);
        self.fifo.clock.next = self.clock.val();
        self.buffer.buf_in_write_enable.next = self.fifo.write_enable.val();
    }
}

#[cfg(test)]
impl FIFOSDRAMTest {
    pub fn new(cas_latency: u32, timings: MemoryTimings, buffer: OutputBuffer) -> Self {
        Self {
            dram: SDRAMSimulator::new(timings.clone()),
            buffer: Default::default(),
            fifo: SDRAMFIFOController::new(cas_latency, timings, buffer),
            clock: Default::default(),
        }
    }
}

#[cfg(test)]
fn make_test_fifo_controller() -> FIFOSDRAMTest {
    let timings = MemoryTimings::fast_boot_sim(100e6);
    let mut uut = FIFOSDRAMTest::new(3, timings, OutputBuffer::DelayOne);
    uut.fifo.write.connect();
    uut.fifo.data_in.connect();
    uut.fifo.read.connect();
    uut.clock.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_sdram_fifo_synthesizes() {
    let uut = make_test_fifo_controller();
    yosys_validate("sdram_fifo_controller", &generate_verilog(&uut)).unwrap();
}

#[test]
fn test_sdram_works() {
    let uut = make_test_fifo_controller();
    let mut sim = Simulation::new();
    sim.add_clock(5000, |x: &mut Box<FIFOSDRAMTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<FIFOSDRAMTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for counter in 0_u32..128_u32 {
            x = sim.watch(|x| !x.fifo.full.val(), x)?;
            x.fifo.data_in.next = counter.into();
            x.fifo.write.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.write.next = false;
        }
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<FIFOSDRAMTest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, clock, x);
        for counter in 0_u32..128_u32 {
            x = sim.watch(|x| !x.fifo.empty.val(), x)?;
            sim_assert_eq!(sim, x.fifo.data_out.val(), counter, x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
        }
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 100_000_000, "fifo_sdram.vcd")
        .unwrap();
}
