use rust_hdl_bsp_alchitry_cu::synth::generate_bitstream;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;
use std::time::Duration;
use rust_hdl_test_core::target_path;

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
        let pulser = Pulser::new(MHZ100, 1.0, Duration::from_millis(250));
        Self {
            pulser,
            clock: rust_hdl_bsp_alchitry_cu::pins::clock(),
            leds: rust_hdl_bsp_alchitry_cu::pins::leds(),
        }
    }
}

#[test]
fn synthesize_alchitry_cu_pulser() {
    let uut = AlchitryCuPulser::default();
    generate_bitstream(uut, target_path!("alchitry_cu/pulser"));
}
