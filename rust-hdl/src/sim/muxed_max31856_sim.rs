use super::max31856_sim::MAX31856Simulator;
use crate::core::prelude::*;
use crate::widgets::prelude::SPIConfig;

#[derive(LogicBlock)]
pub struct MuxedMAX31856Simulators {
    // Input SPI bus
    pub mosi: Signal<In, Bit>,
    pub mclk: Signal<In, Bit>,
    pub msel: Signal<In, Bit>,
    pub miso: Signal<Out, Bit>,
    pub addr: Signal<In, Bits<3>>,
    pub clock: Signal<In, Clock>,
    adcs: [MAX31856Simulator; 8],
}

impl MuxedMAX31856Simulators {
    pub fn new(config: SPIConfig) -> Self {
        Self {
            mosi: Default::default(),
            mclk: Default::default(),
            msel: Default::default(),
            miso: Default::default(),
            addr: Default::default(),
            clock: Default::default(),
            adcs: [
                MAX31856Simulator::new(config),
                MAX31856Simulator::new(config),
                MAX31856Simulator::new(config),
                MAX31856Simulator::new(config),
                MAX31856Simulator::new(config),
                MAX31856Simulator::new(config),
                MAX31856Simulator::new(config),
                MAX31856Simulator::new(config),
            ],
        }
    }
}

impl Logic for MuxedMAX31856Simulators {
    #[hdl_gen]
    fn update(&mut self) {
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
    use rust_hdl_yosys_synth::yosys_validate;
    use crate::prelude::AD7193Config;
    let mut uut = MuxedMAX31856Simulators::new(AD7193Config::hw().spi);
    uut.mclk.connect();
    uut.mosi.connect();
    uut.msel.connect();
    uut.addr.connect();
    uut.clock.connect();
    uut.connect_all();
    yosys_validate("mux_31865", &generate_verilog(&uut)).unwrap();
}
