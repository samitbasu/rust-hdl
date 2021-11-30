use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
use std::time::Duration;

const MHZ100: u64 = 100_000_000;

#[derive(LogicBlock)]
pub struct AlchitryCuPulser {
    pulser: Pulser,
    clock: Signal<In, Clock>,
    leds: Signal<Out, Bits<8>>,
}

impl Logic for AlchitryCuPulser {
    #[hdl_gen]
    fn update(&mut self) {
        self.pulser.enable.next = true;
        self.pulser.clock.next = self.clock.val();
        self.leds.next = 0x00_u32.into();
        if self.pulser.pulse.val() {
            self.leds.next = 0xAA_u32.into();
        }
    }
}

impl Default for AlchitryCuPulser {
    fn default() -> Self {
        let pulser = Pulser::new(
            rust_hdl::bsp::alchitry_cu::pins::CLOCK_SPEED_100MHZ,
            1.0,
            Duration::from_millis(250),
        );
        Self {
            pulser,
            clock: rust_hdl::bsp::alchitry_cu::pins::clock(),
            leds: rust_hdl::bsp::alchitry_cu::pins::leds(),
        }
    }
}

#[test]
fn synthesize_alchitry_cu_pulser() {
    let uut = AlchitryCuPulser::default();
    rust_hdl::bsp::alchitry_cu::synth::generate_bitstream(uut, target_path!("alchitry_cu/pulser"));
}
