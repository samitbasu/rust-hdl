use rust_hdl_alchitry_cu::synth::generate_bitstream;
use rust_hdl_core::prelude::*;
use crate::pulser::Pulser;
use rust_hdl_alchitry_cu::ice_pll::ICE40PLLBlock;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct AlchitryCuPulserPLL {
    pulser: Pulser<25_000_000>,
    clock: Signal<In, Clock<100_000_000>>,
    leds: Signal<Out, Bits<8>>,
    pll: ICE40PLLBlock<100_000_000, 25_000_000>,
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
        let pulser = Pulser::new(1.0, Duration::from_millis(100));
        Self {
            pulser,
            clock: rust_hdl_alchitry_cu::pins::clock(),
            leds: rust_hdl_alchitry_cu::pins::leds(),
            pll: ICE40PLLBlock::default(),
        }
    }
}

#[test]
fn synthesize_alchitry_cu_pulser_with_pll() {
    let uut = AlchitryCuPulserPLL::default();
    generate_bitstream(uut, "pulser_pll");
}
