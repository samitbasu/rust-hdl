use rust_hdl::prelude::*;
use rust_hdl_lib_bsp_alchitry_cu::{pins, synth};
use std::collections::BTreeMap;

#[derive(LogicBlock)]
pub struct AlchitryCuPWM<const P: usize> {
    pwm: PulseWidthModulator<P>,
    clock: Signal<In, Clock>,
    strobe: Strobe<32>,
    leds: Signal<Out, Bits<8>>,
    rom: ROM<Bits<P>, 8>,
    counter: DFF<Bits<8>>,
}

impl<const P: usize> Logic for AlchitryCuPWM<P> {
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

impl<const P: usize> AlchitryCuPWM<P> {
    fn new(clock_freq: u64) -> Self {
        let rom = (0..256)
            .map(|x| (x.to_bits(), snore(x)))
            .collect::<BTreeMap<_, _>>();
        Self {
            pwm: PulseWidthModulator::default(),
            clock: pins::clock(),
            strobe: Strobe::new(clock_freq, 60.0),
            leds: pins::leds(),
            rom: ROM::new(rom),
            counter: Default::default(),
        }
    }
}

#[test]
fn test_pwm_synthesizes() {
    let mut uut: AlchitryCuPWM<6> = AlchitryCuPWM::new(100_000_000);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("pwm_cu2", &vlog).unwrap();
    synth::generate_bitstream(uut, target_path!("alchitry_cu/pwm_cu2"));
}
