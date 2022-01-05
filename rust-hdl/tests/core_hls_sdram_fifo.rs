mod test_common;

use crate::test_common::fifo_tester::{LazyFIFOFeeder, LazyFIFOReader};
use rand::Rng;
use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::hls::sdram_fifo::SDRAMFIFO;
use rust_hdl::sim::sdr_sdram::chip::SDRAMSimulator;
use rust_hdl::widgets::prelude::MemoryTimings;

#[derive(LogicBlock)]
struct HLSSDRAMFIFOTest {
    fifo: SDRAMFIFO<5, 5, 12, 16>,
    sdram: SDRAMSimulator<16>,
    clock: Signal<In, Clock>,
}

impl Default for HLSSDRAMFIFOTest {
    fn default() -> Self {
        let timings = MemoryTimings::fast_boot_sim(125e6);
        Self {
            fifo: SDRAMFIFO::new(3, timings),
            sdram: SDRAMSimulator::new(timings),
            clock: Default::default(),
        }
    }
}

impl Logic for HLSSDRAMFIFOTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.fifo.clock.next = self.clock.val();
        self.sdram.clock.next = self.clock.val();
        self.sdram.address.next = self.fifo.address.val();
        self.sdram.bank.next = self.fifo.bank.val();
        self.sdram.cmd.next = self.fifo.cmd.val();
        self.sdram.data.join(&mut self.fifo.data);
    }
}

#[test]
fn test_hls_sdram_fifo_synthesizes() {
    let mut uut = HLSSDRAMFIFOTest::default();
    uut.clock.connect();
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
    uut.fifo.bus_write.link_connect_dest();
    uut.fifo.bus_read.link_connect_dest();
    uut.connect_all();
    let mut sim = Simulation::new();
    let data = (0..1256)
        .map(|x| rand::thread_rng().gen::<u16>())
        .collect::<Vec<_>>();
    let data2 = data.clone();
    sim.add_clock(4000, |x: &mut Box<HLSSDRAMFIFOTest>| {
        x.clock.next = !x.clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<HLSSDRAMFIFOTest>| {
        let mut x = sim.init()?;
        hls_fifo_write_lazy!(sim, clock, x, fifo.bus_write, &data);
        sim.done(x)
    });
    sim.add_testbench(move |mut sim: Sim<HLSSDRAMFIFOTest>| {
        let mut x = sim.init()?;
        hls_fifo_read_lazy!(sim, clock, x, fifo.bus_read, &data2);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 80_000_000, &vcd_path!("hls_sdram_fifo.vcd"))
        .unwrap();
}
