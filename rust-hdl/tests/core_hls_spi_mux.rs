use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct SPIMuxTest {
    pc_to_host: SyncFIFO<Bits<8>, 3, 4, 1>,
    host_to_pc: SyncFIFO<Bits<8>, 3, 4, 1>,
    bidi_dev: BidiSimulatedDevice<Bits<8>>,
    host: Host<8>,
    route: Router<16, 8, 3>,
    core_1: HLSSPIMaster<16, 8, 64>,
    core_2: HLSSPIMaster<16, 8, 64>,
    mux: HLSSPIMuxMasters<16, 8, 2>,
    pub bidi_clock: Signal<In, Clock>,
    pub sys_clock: Signal<In, Clock>,
    pub spi: SPIWiresMaster,
}

impl HLSNamedPorts for SPIMuxTest {
    fn ports(&self) -> Vec<String> {
        self.route.ports()
    }
}

impl Logic for SPIMuxTest {
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
        SoCBusController::<16, 8>::join(&mut self.host.bus, &mut self.route.upstream);
        SoCBusController::<16, 8>::join(&mut self.route.nodes[0], &mut self.core_1.upstream);
        SoCBusController::<16, 8>::join(&mut self.route.nodes[1], &mut self.core_2.upstream);
        SoCBusController::<16, 8>::join(&mut self.route.nodes[2], &mut self.mux.upstream);
        SPIWiresMaster::join(&mut self.core_1.spi, &mut self.mux.from_masters[0]);
        SPIWiresMaster::join(&mut self.core_2.spi, &mut self.mux.from_masters[1]);
        SPIWiresMaster::link(&mut self.spi, &mut self.mux.to_bus);
        self.mux.to_bus.miso.next = !self.mux.to_bus.mosi.val(); // Echo...
    }
}

impl Default for SPIMuxTest {
    fn default() -> Self {
        let spi_config_1 = SPIConfig {
            clock_speed: 100_000_000,
            cs_off: true,
            mosi_off: false,
            speed_hz: 5_000_000,
            cpha: true,
            cpol: false,
        };
        let spi_config_2 = SPIConfig {
            clock_speed: 100_000_000,
            cs_off: false,
            mosi_off: false,
            speed_hz: 10_000_000,
            cpha: false,
            cpol: false,
        };
        let core_1 = HLSSPIMaster::new(spi_config_1);
        let core_2 = HLSSPIMaster::new(spi_config_2);
        let mux = HLSSPIMuxMasters::default();
        Self {
            pc_to_host: Default::default(),
            host_to_pc: Default::default(),
            bidi_dev: Default::default(),
            host: Default::default(),
            route: Router::new(["spi1", "spi2", "mux"], [&core_1, &core_2, &mux]),
            core_1,
            core_2,
            bidi_clock: Default::default(),
            sys_clock: Default::default(),
            spi: Default::default(),
            mux,
        }
    }
}

#[cfg(test)]
fn make_spi_mux_test() -> SPIMuxTest {
    let mut uut = SPIMuxTest::default();
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
fn test_spi_mux_test_synthesizes() {
    let uut = make_spi_mux_test();
    let vlog = generate_verilog(&uut);
    yosys_validate("spi_mux_test", &vlog).unwrap();
}

#[test]
fn test_spi_mux_test_map() {
    let uut = make_spi_mux_test();
    println!("{:?}", uut.ports());
    assert_eq!(
        uut.ports(),
        [
            "spi1_data_outbound",
            "spi1_data_inbound",
            "spi1_num_bits",
            "spi1_start_flag",
            "spi2_data_outbound",
            "spi2_data_inbound",
            "spi2_num_bits",
            "spi2_start_flag",
            "mux_select"
        ]
    );
}

#[test]
fn test_spi_mux_works() {
    let uut = make_spi_mux_test();
    let mut sim = Simulation::new();
    sim.add_clock(5, |x: &mut Box<SPIMuxTest>| {
        x.bidi_clock.next = !x.bidi_clock.val()
    });
    sim.add_clock(4, |x: &mut Box<SPIMuxTest>| {
        x.sys_clock.next = !x.sys_clock.val()
    });
    sim.add_testbench(move |mut sim: Sim<SPIMuxTest>| {
        let mut x = sim.init()?;
        let ports = x.ports();
        let spi1_data_outbound = ports
            .iter()
            .position(|x| x == "spi1_data_outbound")
            .unwrap();
        let spi1_data_inbound = ports.iter().position(|x| x == "spi1_data_inbound").unwrap();
        let spi1_num_bits = ports.iter().position(|x| x == "spi1_num_bits").unwrap();
        let spi1_start_flag = ports.iter().position(|x| x == "spi1_start_flag").unwrap();
        let spi2_data_outbound = ports
            .iter()
            .position(|x| x == "spi2_data_outbound")
            .unwrap();
        let spi2_data_inbound = ports.iter().position(|x| x == "spi2_data_inbound").unwrap();
        let spi2_num_bits = ports.iter().position(|x| x == "spi2_num_bits").unwrap();
        let spi2_start_flag = ports.iter().position(|x| x == "spi2_start_flag").unwrap();
        let mux_select = ports.iter().position(|x| x == "mux_select").unwrap();
        println!("{:?}", ports);
        wait_clock_true!(sim, bidi_clock, x);
        // Write the outgoing word
        hls_host_write!(
            sim,
            bidi_clock,
            x,
            pc_to_host,
            spi1_data_outbound,
            [0_u16, 0, 0xDEAD_u16, 0xBEEF]
        );
        // Write the transaction length
        hls_host_write!(sim, bidi_clock, x, pc_to_host, spi1_num_bits, [32_u16]);
        // Write a start to start the transaction
        hls_host_write!(sim, bidi_clock, x, pc_to_host, spi1_start_flag, [0_u16]);
        // Read back the results
        hls_host_issue_read!(sim, bidi_clock, x, pc_to_host, spi1_data_inbound, 4);
        let ret = hls_host_get_words!(sim, bidi_clock, x, host_to_pc, 4);
        wait_clock_cycle!(sim, bidi_clock, x, 10);
        sim_assert!(sim, ret == [0x0, 0x0, !0xDEAD_u16, !0xBEEF_u16], x);
        // Switch to the second controller
        hls_host_write!(sim, bidi_clock, x, pc_to_host, mux_select, [1_u16]);
        // Write the outgoing word, this time to the second SPI controller
        hls_host_write!(
            sim,
            bidi_clock,
            x,
            pc_to_host,
            spi2_data_outbound,
            [0_u16, 0, 0xCAFE_u16, 0xBABE]
        );
        // Write the transaction length
        hls_host_write!(sim, bidi_clock, x, pc_to_host, spi2_num_bits, [32_u16]);
        // Write a start to start the transaction
        hls_host_write!(sim, bidi_clock, x, pc_to_host, spi2_start_flag, [0_u16]);
        // Read back the results
        hls_host_issue_read!(sim, bidi_clock, x, pc_to_host, spi2_data_inbound, 4);
        let ret = hls_host_get_words!(sim, bidi_clock, x, host_to_pc, 4);
        sim_assert!(sim, ret == [0x0, 0x0, !0xCAFE_u16, !0xBABE_u16], x);
        wait_clock_cycle!(sim, bidi_clock, x, 100);
        sim.done(x)
    });
    sim.run(Box::new(uut), 10_000).unwrap();
}
