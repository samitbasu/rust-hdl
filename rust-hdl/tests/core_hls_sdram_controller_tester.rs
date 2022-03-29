use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::hls::sdram_controller_tester::SDRAMControllerTester;
use rust_hdl::sim::sdr_sdram::chip::SDRAMSimulator;
use rust_hdl::widgets::prelude::MemoryTimings;
use rust_hdl::widgets::sdram::basic_controller::OutputBuffer;
use rust_hdl::widgets::sdram::buffer::SDRAMOnChipBuffer;
use rust_hdl::widgets::sdram::SDRAMDriver;

#[derive(LogicBlock)]
struct HostSDRAMControllerFixture {
    pc_to_host: SyncFIFO<Bits<8>, 3, 4, 1>,
    host_to_pc: SyncFIFO<Bits<8>, 3, 4, 1>,
    bidi_dev: BidiSimulatedDevice<Bits<8>>,
    host: Host<8>,
    core: SDRAMControllerTester<5, 5>,
    buffer: SDRAMOnChipBuffer<16>,
    chip: SDRAMSimulator<5, 5, 10, 16>,
    pub bidi_clock: Signal<In, Clock>,
    pub sys_clock: Signal<In, Clock>,
}

impl Logic for HostSDRAMControllerFixture {
    #[hdl_gen]
    fn update(&mut self) {
        FIFOReadController::<Bits<8>>::join(
            &mut self.bidi_dev.data_to_bus,
            &mut self.pc_to_host.bus_read,
        );
        FIFOWriteController::<Bits<8>>::join(
            &mut self.bidi_dev.data_from_bus,
            &mut self.host_to_pc.bus_write,
        );
        self.host_to_pc.clock.next = self.bidi_clock.val();
        self.pc_to_host.clock.next = self.bidi_clock.val();
        self.bidi_dev.clock.next = self.bidi_clock.val();
        BidiBusD::<Bits<8>>::join(&mut self.bidi_dev.bus, &mut self.host.bidi_bus);
        self.host.bidi_clock.next = self.bidi_clock.val();
        self.host.sys_clock.next = self.sys_clock.val();
        SoCBusController::<16, 8>::join(&mut self.host.bus, &mut self.core.upstream);
        SDRAMDriver::<16>::join(&mut self.core.dram, &mut self.buffer.buf_in);
        SDRAMDriver::<16>::join(&mut self.buffer.buf_out, &mut self.chip.sdram);
    }
}

impl Default for HostSDRAMControllerFixture {
    fn default() -> Self {
        let timings = MemoryTimings::fast_boot_sim(100e6);
        Self {
            pc_to_host: Default::default(),
            host_to_pc: Default::default(),
            bidi_dev: Default::default(),
            host: Default::default(),
            core: SDRAMControllerTester::new(3, timings, OutputBuffer::DelayOne),
            buffer: Default::default(),
            chip: SDRAMSimulator::new(timings),
            bidi_clock: Default::default(),
            sys_clock: Default::default(),
        }
    }
}

#[cfg(test)]
fn make_sdram_test() -> HostSDRAMControllerFixture {
    let mut uut = HostSDRAMControllerFixture::default();
    uut.sys_clock.connect();
    uut.bidi_clock.connect();
    uut.pc_to_host.bus_write.data.connect();
    uut.pc_to_host.bus_write.write.connect();
    uut.host_to_pc.bus_read.read.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_sdram_test_synthesizes() {
    let uut = make_sdram_test();
    let vlog = generate_verilog(&uut);
    yosys_validate("sdram_tester_test", &vlog).unwrap();
}

#[test]
fn test_sdram_controller_tester() {
    let uut = make_sdram_test();
    let mut sim = Simulation::new();
    sim.add_clock(5_000, |x: &mut Box<HostSDRAMControllerFixture>| {
        x.sys_clock.next = !x.sys_clock.val();
    });
    sim.add_clock(10_000, |x: &mut Box<HostSDRAMControllerFixture>| {
        x.bidi_clock.next = !x.bidi_clock.val();
    });
    sim.add_testbench(move |mut sim: Sim<HostSDRAMControllerFixture>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, bidi_clock, x);
        hls_host_ping!(sim, bidi_clock, x, pc_to_host, 0x67);
        let reply = hls_host_get_word!(sim, bidi_clock, x, host_to_pc);
        sim_assert_eq!(sim, reply, 0x167, x);
        // Write the number of cycles to the count
        hls_host_write!(sim, bidi_clock, x, pc_to_host, 0, [0x0000, 0x0010]);
        // Write the 1 to trigger the cycle
        hls_host_write!(sim, bidi_clock, x, pc_to_host, 1, [0x0000]);
        // Read the error count back
        hls_host_issue_read!(sim, bidi_clock, x, pc_to_host, 2, 0x0002);
        let vals = hls_host_get_words!(sim, bidi_clock, x, host_to_pc, 2);
        sim_assert_eq!(sim, vals, [0x0000, 0x0000], x);
        wait_clock_cycles!(sim, bidi_clock, x, 100);
        sim.done(x)
    });
    sim.run_to_file(
        Box::new(uut),
        10_000_000,
        &vcd_path!("hls_sdram_tester_controller.vcd"),
    )
    .unwrap();
}
