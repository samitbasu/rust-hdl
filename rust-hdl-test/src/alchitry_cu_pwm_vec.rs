use std::collections::BTreeMap;

use rust_hdl_core::check_connected::check_connected;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::prelude::*;

use crate::snore;
use rust_hdl_alchitry_cu::pins::Mhz100;

#[derive(LogicBlock)]
pub struct Fader<F: Domain> {
    pub clock: Signal<In, Clock, F>,
    pub active: Signal<Out, Bit, F>,
    pub enable: Signal<In, Bit, F>,
    strobe: Strobe<F, 32>,
    pwm: PulseWidthModulator<F, 6>,
    rom: ROM<Bits<8>, Bits<6>, F>,
    counter: DFF<Bits<8>, F>,
}

impl<F: Domain> Fader<F> {
    pub fn new(phase: u32) -> Self {
        let rom = (0..256_u32)
            .map(|x| (Bits::<8>::from(x), snore::snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            clock: Signal::default(),
            active: Signal::new_with_default(false),
            enable: Signal::default(),
            strobe: Strobe::new(120.0),
            pwm: PulseWidthModulator::default(),
            rom: ROM::new(rom),
            counter: DFF::new(Bits::<8>::default()),
        }
    }
}

impl<F: Domain> Logic for Fader<F> {
    #[hdl_gen]
    fn update(&mut self) {
        self.strobe.clock.next = self.clock.val();
        self.pwm.clock.next = self.clock.val();
        self.counter.clk.next = self.clock.val();
        self.rom.address.next = self.counter.q.val();
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
        self.strobe.enable.next = self.enable.val();
        self.pwm.enable.next = self.enable.val();
        self.active.next = self.pwm.active.val();
        self.pwm.threshold.next = self.rom.data.val();
    }
}

#[derive(LogicBlock)]
pub struct AlchitryCuPWMVec<F: Domain, const P: usize> {
    clock: Signal<In, Clock, F>,
    leds: Signal<Out, Bits<8>, Async>,
    local: Signal<Local, Bits<8>, Async>,
    faders: [Fader<F>; 8],
}

impl<F: Domain, const P: usize> Logic for AlchitryCuPWMVec<F, P> {
    #[hdl_gen]
    fn update(&mut self) {
        for i in 0_usize..8_usize {
            self.faders[i].clock.next = self.clock.val();
            self.faders[i].enable.next = true.into();
        }
        self.local.next = 0x00_u8.into();
        for i in 0_usize..8_usize {
            self.local.next = self
                .local
                .val()
                .raw()
                .replace_bit(i, self.faders[i].active.val().raw())
                .into();
        }
        self.leds.next = self.local.val();
    }
}

impl<const P: usize> Default for AlchitryCuPWMVec<Mhz100, P> {
    fn default() -> Self {
        let faders: [Fader<Mhz100>; 8] = [
            Fader::new(0),
            Fader::new(18),
            Fader::new(36),
            Fader::new(54),
            Fader::new(72),
            Fader::new(90),
            Fader::new(108),
            Fader::new(128),
        ];
        Self {
            clock: rust_hdl_alchitry_cu::pins::clock(),
            leds: rust_hdl_alchitry_cu::pins::leds(),
            local: Signal::default(),
            faders,
        }
    }
}

#[test]
fn test_pwm_vec_synthesizes() {
    let mut uut: AlchitryCuPWMVec<Mhz100, 6> = AlchitryCuPWMVec::default();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("pwm_cu", &vlog).unwrap();
    rust_hdl_alchitry_cu::synth::generate_bitstream(uut, "pwm_cu");
}
