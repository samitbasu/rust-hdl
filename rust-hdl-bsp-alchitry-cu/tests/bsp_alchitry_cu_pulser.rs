use rust_hdl::prelude::*;
use rust_hdl_bsp_alchitry_cu::pins::CLOCK_SPEED_100MHZ;
use rust_hdl_bsp_alchitry_cu::{pins, synth};
use std::time::Duration;

pub const MHZ100: u64 = 100_000_000;

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
        clock!(self, clock, pulser);
        self.leds.next = 0x00.into();
        if self.pulser.pulse.val() {
            self.leds.next = 0xAA.into();
        }
    }
}

impl Default for AlchitryCuPulser {
    fn default() -> Self {
        let pulser = Pulser::new(CLOCK_SPEED_100MHZ.into(), 1.0, Duration::from_millis(250));
        Self {
            pulser,
            clock: pins::clock(),
            leds: pins::leds(),
        }
    }
}

#[test]
fn synthesize_alchitry_cu_pulser() {
    let uut = AlchitryCuPulser::default();
    synth::generate_bitstream(uut, target_path!("alchitry_cu/pulser"));
}
