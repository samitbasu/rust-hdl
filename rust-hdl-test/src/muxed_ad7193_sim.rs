use crate::ad7193_sim::{AD7193Config, AD7193Simulator};
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;

#[derive(LogicBlock)]
pub struct MuxedAD7193Simulators {
    // Input SPI bus
    pub mosi: Signal<In, Bit>,
    pub mclk: Signal<In, Bit>,
    pub msel: Signal<In, Bit>,
    pub miso: Signal<Out, Bit>,
    pub addr: Signal<In, Bits<3>>,
    pub clock: Signal<In, Clock>,
    adcs: [AD7193Simulator; 8],
}

impl MuxedAD7193Simulators {
    pub fn new(config: AD7193Config) -> Self {
        Self {
            mosi: Default::default(),
            mclk: Default::default(),
            msel: Default::default(),
            miso: Default::default(),
            addr: Default::default(),
            clock: Default::default(),
            adcs: [
                AD7193Simulator::new(config),
                AD7193Simulator::new(config),
                AD7193Simulator::new(config),
                AD7193Simulator::new(config),
                AD7193Simulator::new(config),
                AD7193Simulator::new(config),
                AD7193Simulator::new(config),
                AD7193Simulator::new(config),
            ],
        }
    }
}

impl Logic for MuxedAD7193Simulators {
    #[hdl_gen]
    fn update(&mut self) {
        // Latch prevention
        self.miso.next = true;
        for i in 0_usize..8_usize {
            self.adcs[i].clock.next = self.clock.val();
            self.adcs[i].mosi.next = self.mosi.val();
            self.adcs[i].mclk.next = self.mclk.val();
            self.adcs[i].msel.next = true;
            if self.addr.val().index() == i {
                self.adcs[i].msel.next = self.msel.val();
                self.miso.next = self.adcs[i].miso.val();
            }
        }
    }
}

#[test]
fn test_mux_is_synthesizable() {
    let mut uut = MuxedAD7193Simulators::new(AD7193Config::hw());
    uut.mclk.connect();
    uut.mosi.connect();
    uut.msel.connect();
    uut.addr.connect();
    uut.clock.connect();
    uut.connect_all();
    println!("{}", generate_verilog(&uut));
    yosys_validate("mux_7193", &generate_verilog(&uut)).unwrap();
}
