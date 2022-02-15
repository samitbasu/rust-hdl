use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct SPITest {
    pc_to_host: SyncFIFO<Bits<8>, 3, 4, 1>,
    host_to_pc: SyncFIFO<Bits<8>, 3, 4, 1>,
    bidi_dev: BidiSimulatedDevice<Bits<8>>,
    host: Host<8>,
    core: HLSSPIMasterDynamicMode<16, 8, 64>,
    pub bidi_clock: Signal<In, Clock>,
    pub sys_clock: Signal<In, Clock>,
    pub spi: SPIWiresMaster,
}

impl Logic for SPITest {
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
        SPIWiresMaster::link(&mut self.spi, &mut self.core.spi);
    }
}

impl Default for SPITest {
    fn default() -> Self {
        let spi_config = SPIConfigDynamicMode {
            clock_speed: 100_000_000,
            cs_off: true,
            mosi_off: false,
            speed_hz: 10_000_000,
        };
        Self {
            pc_to_host: Default::default(),
            host_to_pc: Default::default(),
            bidi_dev: Default::default(),
            host: Default::default(),
            core: HLSSPIMasterDynamicMode::new(spi_config),
            bidi_clock: Default::default(),
            sys_clock: Default::default(),
            spi: Default::default(),
        }
    }
}

#[cfg(test)]
fn make_spi_test() -> SPITest {
    let mut uut = SPITest::default();
    uut.sys_clock.connect();
    uut.bidi_clock.connect();
    uut.pc_to_host.bus_write.data.connect();
    uut.pc_to_host.bus_write.write.connect();
    uut.host_to_pc.bus_read.read.connect();
    uut.spi.miso.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_spi_test_dynamic_mode_synthesizes() {
    let uut = make_spi_test();
    let vlog = generate_verilog(&uut);
    yosys_validate("spi_dm_test", &vlog).unwrap();
}

#[test]
fn test_spi_dynamic_mode_works() {
    let uut = make_spi_test();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SPITest>| {
        x.bidi_clock.next = !x.bidi_clock.val()
    });
    sim.add_clock(4, |x: &mut Box<SPITest>| {
        x.sys_clock.next = !x.sys_clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SPITest>| {
        let mut x = sim.init()?;
        wait_clock_true!(sim, bidi_clock, x);
        // Write the outgoing word
        hls_host_write!(
            sim,
            bidi_clock,
            x,
            pc_to_host,
            0,
            [0_u16, 0, 0xDEAD_u16, 0xBEEF]
        );
        // Write the transaction length - set the SPI mode to 2, which
        // sets the cpol and cpha bits to bits 8 and 9 of the length register
        // shifted by 8 bits relative to the length
        let mode_and_length: u16 = 32 | (3 << 8);
        hls_host_write!(sim, bidi_clock, x, pc_to_host, 2, [mode_and_length]);
        // Write a start to start the transaction
        hls_host_write!(sim, bidi_clock, x, pc_to_host, 3, [0_u16]);
        // Read back the results
        hls_host_issue_read!(sim, bidi_clock, x, pc_to_host, 1, 4);
        let ret = hls_host_get_words!(sim, bidi_clock, x, host_to_pc, 4);
        wait_clock_cycle!(sim, bidi_clock, x, 100);
        println!("{:x?}", ret);
        sim.done(x)
    });
    sim.run_to_file(Box::new(uut), 100_000, &vcd_path!("host_spi_dm.vcd"))
        .unwrap();
}
