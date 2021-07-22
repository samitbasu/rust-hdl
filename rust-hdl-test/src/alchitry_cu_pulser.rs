use rust_hdl_alchitry_cu::synth::generate_bitstream;
use rust_hdl_core::prelude::*;
use crate::pulser::Pulser;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct AlchitryCuPulser {
    pulser: Pulser<100_000_000>,
    clock: Signal<In, Clock<100_000_000>>,
    leds: Signal<Out, Bits<8>>,
}

impl Logic for AlchitryCuPulser {
    #[hdl_gen]
    fn update(&mut self) {
        self.pulser.enable.next = true;
        self.pulser.clock.next = self.clock.val();
        self.leds.next = 0x00_u8.into();
        if self.pulser.pulse.val() {
            self.leds.next = 0xAA_u8.into();
        }
    }
}

impl Default for AlchitryCuPulser {
    fn default() -> Self {
        let pulser = Pulser::new(1.0, Duration::from_millis(250));
        Self {
            pulser,
            clock: rust_hdl_alchitry_cu::pins::clock(),
            leds: rust_hdl_alchitry_cu::pins::leds(),
        }
    }
}

#[test]
fn synthesize_alchitry_cu_pulser() {
    let uut = AlchitryCuPulser::default();
    generate_bitstream(uut, "pulser");
}
