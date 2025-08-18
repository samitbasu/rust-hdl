use rust_hdl::prelude::*;
use rust_hdl_bsp_orange_crab::{pins, synth};
use std::time::Duration;

#[derive(LogicBlock)]
pub struct OrangeCrabPulser {
    pulser: Pulser,
    clock: Signal<In, Clock>,
    leds: Signal<Out, Bits<3>>,
}

impl Logic for OrangeCrabPulser {
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

impl Default for OrangeCrabPulser {
    fn default() -> Self {
        let pulser = Pulser::new(pins::CLOCK_SPEED.into(), 1.0, Duration::from_millis(250));

        Self {
            pulser,
            clock: pins::Pin::clock(),
            leds: pins::Pin::leds(),
        }
    }
}

#[test]
fn synthesize_orange_crab_pulser() {
    [synth::Density::_25F, synth::Density::_85F]
        .into_iter()
        .for_each(|density| {
            let uut = OrangeCrabPulser::default();

            synth::generate_bitstream(
                uut,
                target_path!(format!("orange_crab/pulser_{density}")),
                density,
            );
        });
}
