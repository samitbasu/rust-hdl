use rust_hdl::core::prelude::*;
use rust_hdl::hls::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct SPIMuxSlavesTest {
    pc_to_host: SyncFIFO<Bits<8>, 3, 4, 1>,
    host_to_pc: SyncFIFO<Bits<8>, 3, 4, 1>,
    bidi_dev: BidiSimulatedDevice<Bits<8>>,
    host: Host<8>,
    route: Router<16, 8, 2>,
    core: HLSSPIMaster<16, 8, 64>,
    mux: HLSSPIMuxSlaves<16, 8, 2>,
    auto_reset: AutoReset,
    bidi_reset: Signal<Local, Reset>,
    pub bidi_clock: Signal<In, Clock>,
    pub sys_clock: Signal<In, Clock>,
}

impl HLSNamedPorts for SPIMuxSlavesTest {
    fn ports(&self) -> Vec<String> {
        self.route.ports()
    }
}

impl Logic for SPIMuxSlavesTest {
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
        self.auto_reset.clock.next = self.bidi_clock.val();
        self.bidi_reset.next = self.auto_reset.reset.val();
        clock_reset!(self, bidi_clock, bidi_reset, host_to_pc, pc_to_host);
        self.bidi_dev.clock.next = self.bidi_clock.val();
        BidiBusD::<Bits<8>>::join(&mut self.bidi_dev.bus, &mut self.host.bidi_bus);
        self.host.bidi_clock.next = self.bidi_clock.val();
        self.host.sys_clock.next = self.sys_clock.val();
        self.host.reset.next = self.auto_reset.reset.val();
        SoCBusController::<16, 8>::join(&mut self.host.bus, &mut self.route.upstream);
        SoCBusController::<16, 8>::join(&mut self.route.nodes[0], &mut self.core.upstream);
        SoCBusController::<16, 8>::join(&mut self.route.nodes[1], &mut self.mux.upstream);
        SPIWiresMaster::join(&mut self.core.spi, &mut self.mux.from_bus);
        self.mux.to_slaves[0].miso.next = !self.mux.to_slaves[0].mosi.val();
        self.mux.to_slaves[1].miso.next = self.mux.to_slaves[1].miso.val();
    }
}

impl Default for SPIMuxSlavesTest {
    fn default() -> Self {
        let spi_config = SPIConfig {
            clock_speed: 100_000_000,
            cs_off: true,
            mosi_off: false,
            speed_hz: 5_000_000,
            cpha: true,
            cpol: false,
        };
        let core = HLSSPIMaster::new(spi_config);
        let mux = HLSSPIMuxSlaves::default();
        Self {
            pc_to_host: Default::default(),
            host_to_pc: Default::default(),
            bidi_dev: Default::default(),
            host: Default::default(),
            route: Router::new(["spi", "mux"], [&core, &mux]),
            core,
            mux,
            auto_reset: Default::default(),
            bidi_reset: Default::default(),
            bidi_clock: Default::default(),
            sys_clock: Default::default(),
        }
    }
}

#[cfg(test)]
fn make_spi_mux_slaves_test() -> SPIMuxSlavesTest {
    let mut uut = SPIMuxSlavesTest::default();
    uut.pc_to_host.bus_write.data.connect();
    uut.pc_to_host.bus_write.write.connect();
    uut.host_to_pc.bus_read.read.connect();
    uut.connect_all();
    uut
}

#[test]
fn test_spi_mux_slaves_test_synthesizes() {
    let uut = make_spi_mux_slaves_test();
    let vlog = generate_verilog(&uut);
    yosys_validate("spi_mux_slaves_test", &vlog).unwrap();
}

#[test]
fn test_spi_mux_slaves_test_map() {
    let uut = make_spi_mux_slaves_test();
    println!("{:?}", uut.ports());
    assert_eq!(
        uut.ports(),
        [
            "spi_data_outbound",
            "spi_data_inbound",
            "spi_num_bits",
            "spi_start_flag",
            "mux_select"
        ]
    );
}
