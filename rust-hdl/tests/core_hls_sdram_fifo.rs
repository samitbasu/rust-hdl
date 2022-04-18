mod test_common;

use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::hls::sdram_fifo::SDRAMFIFO;
use rust_hdl::sim::sdr_sdram::chip::SDRAMSimulator;
use rust_hdl::widgets::prelude::{MemoryTimings, OutputBuffer};
use rust_hdl::widgets::sdram::SDRAMDriver;

#[derive(LogicBlock)]
struct HLSSDRAMFIFOTest {
    fifo: SDRAMFIFO<5, 5, 4, 16, 12>,
    sdram: SDRAMSimulator<5, 5, 10, 16>,
    clock: Signal<In, Clock>,
    reset: Signal<In, ResetN>,
}

impl Default for HLSSDRAMFIFOTest {
    fn default() -> Self {
        let timings = MemoryTimings::fast_boot_sim(125e6);
        Self {
            fifo: SDRAMFIFO::new(3, timings, OutputBuffer::Wired),
            sdram: SDRAMSimulator::new(timings),
            clock: Default::default(),
            reset: Default::default(),
        }
    }
}

impl Logic for HLSSDRAMFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        clock_reset!(self, clock, reset, fifo);
        self.fifo.ram_clock.next = self.clock.val();
        SDRAMDriver::<16>::join(&mut self.fifo.sdram, &mut self.sdram.sdram);
    }
}

#[test]
fn test_hls_sdram_fifo_synthesizes() {
    let mut uut = HLSSDRAMFIFOTest::default();
    uut.clock.connect();
    uut.reset.connect();
    uut.fifo.bus_write.link_connect_dest();
    uut.fifo.bus_read.link_connect_dest();
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("hls_sdram_fifo", &vlog).unwrap();
}

#[test]
fn test_hls_sdram_fifo_works() {
    let mut uut = HLSSDRAMFIFOTest::default();
    uut.clock.connect();
    uut.reset.connect();
    uut.fifo.bus_write.link_connect_dest();
    uut.fifo.bus_read.link_connect_dest();
    uut.connect_all();
    let mut sim = Simulation::new();
    let data = (0..256)
        .map(|_| rand::thread_rng().gen::<u64>())
        .collect::<Vec<_>>();
    let data2 = data.clone();
    sim.add_clock(4000, |x: &mut Box<HLSSDRAMFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<HLSSDRAMFIFOTest>| {
        let mut x = sim.init()?;
        reset_sim!(sim, clock, reset, x);
        hls_fifo_write_lazy!(sim, clock, x, fifo.bus_write, &data);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<HLSSDRAMFIFOTest>| {
        let mut x = sim.init()?;
        hls_fifo_read_lazy!(sim, clock, x, fifo.bus_read, &data2);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 200_000_000, &vcd_path!("hls_sdram_fifo.vcd"))
        .unwrap();
}
