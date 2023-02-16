use super::ad7193_sim::{AD7193Config, AD7193Simulator};
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;

#[derive(LogicBlock)]
pub struct MuxedAD7193Simulators {
    // Input SPI bus
    pub wires: SPIWiresSlave,
    pub addr: Signal<In, Bits<3>>,
    pub mux: MuxSlaves<8, 3>,
    pub clock: Signal<In, Clock>,
    adcs: [AD7193Simulator; 8],
}

impl MuxedAD7193Simulators {
    pub fn new(config: AD7193Config) -> Self {
        Self {
            wires: Default::default(),
            mux: Default::default(),
            addr: Default::default(),
            clock: Default::default(),
            adcs: array_init::array_init(|_| AD7193Simulator::new(config)),
        }
    }
}

impl Logic for MuxedAD7193Simulators {
    #[hdl_gen]
    fn update(&mut self) {
        SPIWiresSlave::link(&mut self.wires, &mut self.mux.from_master);
        for i in 0..8 {
            self.adcs[i].clock.next = self.clock.val();
            SPIWiresMaster::join(&mut self.mux.to_slaves[i], &mut self.adcs[i].wires);
        }
        self.mux.sel.next = self.addr.val();
    }
}

#[test]
fn test_mux_is_synthesizable() {
    let mut uut = MuxedAD7193Simulators::new(AD7193Config::hw());
    uut.connect_all();
    yosys_validate("mux_7193", &generate_verilog(&uut)).unwrap();
}
