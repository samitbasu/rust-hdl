use std::collections::BTreeMap;

use rust_hdl::core::prelude::*;
mod test_common;
use rust_hdl::widgets::prelude::*;
use test_common::snore;

#[derive(LogicBlock)]
pub struct Fader {
    pub reset: Signal<In, Reset>,
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
        let rom = (0..256_u32)
            .map(|x| (Bits::<8>::from(x), snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            reset: Default::default(),
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
        clock_reset!(self, clock, reset, strobe, pwm);
        dff_setup!(self, clock, reset, counter);
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
    reset: Signal<Local, Reset>,
    auto_reset: AutoReset,
}

impl<const P: usize> Logic for AlchitryCuPWMVec<P> {
    #[hdl_gen]
    fn update(&mut self) {
        self.auto_reset.clock.next = self.clock.val();
        self.reset.next = self.auto_reset.reset.val();
        for i in 0_usize..8_usize {
            self.faders[i].clock.next = self.clock.val();
            self.faders[i].reset.next = self.reset.val();
            self.faders[i].enable.next = true;
        }
        self.local.next = 0x00_u8.into();
        for i in 0_usize..8_usize {
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
            clock: rust_hdl::bsp::alchitry_cu::pins::clock(),
            leds: rust_hdl::bsp::alchitry_cu::pins::leds(),
            local: Signal::default(),
            faders,
            reset: Default::default(),
            auto_reset: Default::default()
        }
    }
}

#[test]
fn test_pwm_vec_synthesizes() {
    let mut uut: AlchitryCuPWMVec<6> = AlchitryCuPWMVec::new(100_000_000);
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    yosys_validate("pwm_cu", &vlog).unwrap();
    rust_hdl::bsp::alchitry_cu::synth::generate_bitstream(uut, target_path!("alchitry_cu/pwm_cu"));
}
