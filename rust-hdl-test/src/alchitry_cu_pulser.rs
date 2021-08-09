use rust_hdl_widgets::pulser::Pulser;
use rust_hdl_alchitry_cu::pins::Mhz100;
use rust_hdl_alchitry_cu::synth::generate_bitstream;
use rust_hdl_core::prelude::*;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct AlchitryCuPulser {
    pulser: Pulser<Mhz100>,
    clock: Signal<In, Clock, Mhz100>,
    leds: Signal<Out, Bits<8>, Async>,
}

impl Logic for AlchitryCuPulser {
    #[hdl_gen]
    fn update(&mut self) {
        self.pulser.enable.next = true.into();
        self.pulser.clock.next = self.clock.val();
        self.leds.next = 0x00_u32.into();
        if self.pulser.pulse.val().raw() {
            self.leds.next = 0xAA_u32.into();
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
