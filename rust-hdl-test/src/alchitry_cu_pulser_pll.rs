use rust_hdl_alchitry_cu::synth::generate_bitstream;
use rust_hdl_core::prelude::*;
use crate::pulser::Pulser;
use rust_hdl_alchitry_cu::ice_pll::ICE40PLLBlock;

#[derive(LogicBlock)]
pub struct AlchitryCuPulserPLL {
    pulser: Pulser,
    clock: Signal<In, Clock>,
    leds: Signal<Out, Bits<8>>,
    pll: ICE40PLLBlock,
}

impl Logic for AlchitryCuPulserPLL {
    #[hdl_gen]
    fn update(&mut self) {
        self.pulser.enable.next = true;
        self.pll.clock_in.next = self.clock.val();
        self.pulser.clock.next = self.pll.clock_out.val();
        self.leds.next = 0x00_u8.into();
        if self.pulser.pulse.val() {
            self.leds.next = 0xAA_u8.into();
        }
    }
}

impl Default for AlchitryCuPulserPLL {
    fn default() -> Self {
        let pulser = Pulser::new(25_000_000, 1, 10_000_000);
        Self {
            pulser,
            clock: rust_hdl_alchitry_cu::pins::clock(),
            leds: rust_hdl_alchitry_cu::pins::leds(),
            pll: ICE40PLLBlock::new(100.0, 25.0)
        }
    }
}

#[test]
fn synthesize_alchitry_cu_pulser_with_pll() {
    let uut = AlchitryCuPulserPLL::default();
    generate_bitstream(uut, "pulser_pll");
}
