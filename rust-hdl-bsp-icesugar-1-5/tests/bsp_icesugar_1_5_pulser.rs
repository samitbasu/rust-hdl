use rust_hdl::prelude::*;
use rust_hdl_bsp_icesugar_1_5::pins::CLOCK_SPEED_12MHZ;
use rust_hdl_bsp_icesugar_1_5::{pins, synth};
use std::time::Duration;

#[derive(LogicBlock)]
pub struct ICESugarPulser {
    pulser: Pulser,
    clock: Signal<In, Clock>,
    r_led: Signal<Out, Bit>,
}

impl Logic for ICESugarPulser {
    #[hdl_gen]
    fn update(&mut self) {
        self.pulser.enable.next = true;
        clock!(self, clock, pulser);
        self.r_led.next = false;
        if self.pulser.pulse.val() {
            self.r_led.next = true;
        }
    }
}

impl Default for ICESugarPulser {
    fn default() -> Self {
        let pulser = Pulser::new(CLOCK_SPEED_12MHZ.into(), 1.0, Duration::from_millis(250));
        Self { 
            pulser, 
            clock: pins::clock(), 
            r_led: pins::red_led() 
        }
    }
}

#[test]
fn synthesize_icesugar_1_5_pulser() {
    let uut = ICESugarPulser::default();
    synth::generate_bitstream(uut, target_path!("icesugar/pulser"));
}
