use std::collections::BTreeMap;

use rust_hdl::prelude::*;
use rust_hdl_lib_bsp_alchitry_cu::{pins, synth};

#[derive(LogicBlock)]
pub struct Fader {
    pub clock: Signal<In, Clock>,
    pub active: Signal<Out, Bit>,
    pub enable: Signal<In, Bit>,
    strobe: Strobe<32>,
    pwm: PulseWidthModulator<6>,
    rom: ROM<Bits<6>, 8>,
    counter: DFF<Bits<8>>,
}

impl Fader {
    pub fn new(clock_frequency: u64, phase: u32) -> Self {
        let rom = (0..256)
            .map(|x| (x.to_bits(), snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            clock: Signal::default(),
            active: Signal::new_with_default(false),
            enable: Signal::default(),
            strobe: Strobe::new(clock_frequency, 120.0),
            pwm: PulseWidthModulator::default(),
            rom: ROM::new(rom),
            counter: Default::default(),
        }
    }
}

impl Logic for Fader {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, strobe, pwm);
        dff_setup!(self, clock, counter);
        self.rom.address.next = self.counter.q.val();
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
        self.strobe.enable.next = self.enable.val();
        self.pwm.enable.next = self.enable.val();
        self.active.next = self.pwm.active.val();
        self.pwm.threshold.next = self.rom.data.val();
    }
}

#[derive(LogicBlock)]
pub struct AlchitryCuPWMVec<const P: usize> {
    clock: Signal<In, Clock>,
    leds: Signal<Out, Bits<8>>,
    local: Signal<Local, Bits<8>>,
    faders: [Fader; 8],
}

impl<const P: usize> Logic for AlchitryCuPWMVec<P> {
    #[hdl_gen]
    fn update(&mut self) {
        for i in 0..8 {
            self.faders[i].clock.next = self.clock.val();
            self.faders[i].enable.next = true;
        }
        self.local.next = 0x00.into();
        for i in 0..8 {
            self.local.next = self.local.val().replace_bit(i, self.faders[i].active.val());
        }
        self.leds.next = self.local.val();
    }
}

impl<const P: usize> AlchitryCuPWMVec<P> {
    fn new(clock_frequency: u64) -> Self {
        let faders: [Fader; 8] = [
            Fader::new(clock_frequency, 0),
            Fader::new(clock_frequency, 18),
            Fader::new(clock_frequency, 36),
            Fader::new(clock_frequency, 54),
            Fader::new(clock_frequency, 72),
            Fader::new(clock_frequency, 90),
            Fader::new(clock_frequency, 108),
            Fader::new(clock_frequency, 128),
        ];
        Self {
            clock: pins::clock(),
            leds: pins::leds(),
            local: Signal::default(),
            faders,
        }
    }
}

#[test]
fn test_pwm_vec_synthesizes() {
    let mut uut: AlchitryCuPWMVec<6> = AlchitryCuPWMVec::new(100_000_000);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("pwm_cu", &vlog).unwrap();
    synth::generate_bitstream(uut, target_path!("alchitry_cu/pwm_cu"));
}
