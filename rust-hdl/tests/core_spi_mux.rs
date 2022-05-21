use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
struct SPITestMultiMaster {
    clock: Signal<In, Clock>,
    masters: [SPIMaster<64>; 3],
    addr: Signal<In, Bits<3>>,
    mux: MuxMasters<3, 3>,
    slave: SPISlave<64>,
}

impl SPITestMultiMaster {
    pub fn new(config: SPIConfig) -> Self {
        Self {
            clock: Default::default(),
            masters: array_init::array_init(|_| SPIMaster::new(config)),
            addr: Default::default(),
            mux: Default::default(),
            slave: SPISlave::new(config),
        }
    }
}

impl Logic for SPITestMultiMaster {
    #[hdl_gen]
    fn update(&mut self) {
        for i in 0_usize..3 {
            self.masters[i].clock.next = self.clock.val();
            SPIWiresMaster::join(&mut self.masters[i].wires, &mut self.mux.from_masters[i]);
        }
        SPIWiresMaster::join(&mut self.mux.to_bus, &mut self.slave.wires);
        clock!(self, clock, slave);
        self.mux.sel.next = self.addr.val();
    }
}

#[test]
fn test_spi_mux() {
    let config = SPIConfig {
        clock_speed: 48_000_000,
        cs_off: true,
        mosi_off: true,
        speed_hz: 1_000_000,
        cpha: true,
        cpol: true,
    };
    let mut uut = SPITestMultiMaster::new(config);
    for i in 0..3 {
        uut.masters[i].continued_transaction.connect();
        uut.masters[i].start_send.connect();
        uut.masters[i].data_outbound.connect();
        uut.masters[i].bits_outbound.connect();
    }
    uut.slave.data_outbound.connect();
    uut.slave.start_send.connect();
    uut.slave.continued_transaction.connect();
    uut.slave.disabled.connect();
    uut.slave.bits.connect();
    uut.connect_all();
    yosys_validate("spi_mux_multi_master", &generate_verilog(&uut)).unwrap();
}
