use super::max31856_sim::MAX31856Simulator;
use crate::core::prelude::*;
use crate::widgets::prelude::{SPIConfig, SPIWiresMaster, SPIWiresSlave};
use crate::widgets::spi::mux::MuxSlaves;

#[derive(LogicBlock)]
pub struct MuxedMAX31856Simulators {
    // Input SPI bus
    pub wires: SPIWiresSlave,
    pub mux: MuxSlaves<8, 3>,
    pub addr: Signal<In, Bits<3>>,
    pub clock: Signal<In, Clock>,
    adcs: [MAX31856Simulator; 8],
}

impl MuxedMAX31856Simulators {
    pub fn new(config: SPIConfig) -> Self {
        Self {
            wires: Default::default(),
            mux: Default::default(),
            addr: Default::default(),
            clock: Default::default(),
            adcs: array_init::array_init(|_| MAX31856Simulator::new(config)),
        }
    }
}

impl Logic for MuxedMAX31856Simulators {
    #[hdl_gen]
    fn update(&mut self) {
        self.wires.link(&mut self.mux.from_master);
        self.mux.sel.next = self.addr.val();
        for i in 0_usize..8_usize {
            self.adcs[i].clock.next = self.clock.val();
            SPIWiresMaster::join(&mut self.mux.to_slaves[i], &mut self.adcs[i].wires);
        }
    }
}

#[test]
fn test_mux_is_synthesizable() {
    use super::ad7193_sim::AD7193Config;
    let mut uut = MuxedMAX31856Simulators::new(AD7193Config::hw().spi);
    uut.wires.link_connect_dest();
    uut.addr.connect();
    uut.clock.connect();
    uut.connect_all();
    yosys_validate("mux_31865", &generate_verilog(&uut)).unwrap();
}
