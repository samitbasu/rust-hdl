use rust_hdl::prelude::*;
use rust_hdl_bsp_orange_crab::{pins, synth};
use std::collections::BTreeMap;

#[derive(LogicBlock)]
pub struct OrangeCrabPWM<const P: usize> {
    pwm: PulseWidthModulator<P>,
    clock: Signal<In, Clock>,
    strobe: Strobe<32>,
    leds: Signal<Out, Bits<3>>,
    rom: ROM<Bits<P>, 8>,
    counter: DFF<Bits<8>>,
}

impl<const P: usize> Logic for OrangeCrabPWM<P> {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, pwm, strobe, counter);
        self.pwm.enable.next = true;
        self.rom.address.next = self.counter.q.val();
        self.pwm.threshold.next = self.rom.data.val();
        self.strobe.enable.next = true;
        self.leds.next = 0x00.into();
        if self.pwm.active.val() {
            self.leds.next = 0xFF.into();
        }
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
    }
}

impl<const P: usize> OrangeCrabPWM<P> {
    fn new(clock_freq: u64) -> Self {
        let rom = (0..256)
            .map(|x| (x.to_bits(), snore(x)))
            .collect::<BTreeMap<_, _>>();

        Self {
            pwm: PulseWidthModulator::default(),
            clock: pins::Pin::clock(),
            strobe: Strobe::new(clock_freq, 60.0),
            leds: pins::Pin::leds(),
            rom: ROM::new(rom),
            counter: Default::default(),
        }
    }
}

#[test]
fn test_pwm_synthesizes() {
    [synth::Density::_25F, synth::Density::_85F]
        .into_iter()
        .for_each(|density| {
            let mut uut: OrangeCrabPWM<6> = OrangeCrabPWM::new(pins::CLOCK_SPEED);
            uut.connect_all();

            let vlog = generate_verilog(&uut);
            yosys_validate("pwm_oc", &vlog).unwrap();

            synth::generate_bitstream(
                uut,
                target_path!(format!("orange_crab/pwm_oc_{density}")),
                density,
            );
        });
}
