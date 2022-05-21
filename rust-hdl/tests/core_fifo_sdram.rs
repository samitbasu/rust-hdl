use rust_hdl::core::prelude::*;
use rust_hdl::sim::sdr_sdram::chip::SDRAMSimulator;
use rust_hdl::vcd_path;
use rust_hdl::widgets::prelude::*;
use rust_hdl::widgets::sdram::buffer::SDRAMOnChipBuffer;

#[cfg(test)]
#[derive(LogicBlock)]
struct FIFOSDRAMTest {
    dram: SDRAMSimulator<6, 4, 10, 16>,
    buffer: SDRAMOnChipBuffer<16>,
    fifo: SDRAMFIFOController<6, 4, 16, 16, 12>,
    clock: Signal<In, Clock>,
}

#[cfg(test)]
impl Logic for FIFOSDRAMTest {
    #[hdl_gen]
    fn update(&mut self) {
        SDRAMDriver::<16>::join(&mut self.fifo.sdram, &mut self.buffer.buf_in);
        SDRAMDriver::<16>::join(&mut self.buffer.buf_out, &mut self.dram.sdram);
        clock!(self, clock, fifo);
        self.fifo.ram_clock.next = self.clock.val();
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
    let mut uut = FIFOSDRAMTest::new(3, timings, OutputBuffer::DelayTwo);
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
        wait_clock_cycles!(sim, clock, x, 20);
        wait_clock_true!(sim, clock, x);
        for counter in 0_u32..512_u32 {
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
        wait_clock_cycles!(sim, clock, x, 20);
        wait_clock_true!(sim, clock, x);
        for counter in 0_u32..512_u32 {
            x = sim.watch(|x| !x.fifo.empty.val(), x)?;
            sim_assert_eq!(sim, x.fifo.data_out.val(), counter, x);
            x.fifo.read.next = true;
            wait_clock_cycle!(sim, clock, x);
            x.fifo.read.next = false;
        }
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 100_000_000, &vcd_path!("fifo_sdram.vcd"))
        .unwrap();
}
