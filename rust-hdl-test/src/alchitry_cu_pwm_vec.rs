use std::collections::BTreeMap;

use rust_hdl_core::check_connected::check_connected;
use rust_hdl_core::prelude::*;
use rust_hdl_synth::yosys_validate;
use rust_hdl_widgets::prelude::*;

use crate::snore;

#[derive(LogicBlock)]
pub struct Fader<const F: u64> {
    pub clock: Signal<In, Clock<F>>,
    pub active: Signal<Out, Bit>,
    pub enable: Signal<In, Bit>,
    strobe: Strobe<32, F>,
    pwm: PulseWidthModulator<6, F>,
    rom: ROM<Bits<8>, Bits<6>>,
    counter: DFF<Bits<8>, F>
}

impl<const F: u64> Fader<F> {
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
            counter: DFF::new(Bits::<8>::default())
        }
    }
}

impl<const F: u64> Logic for Fader<F> {
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
pub struct AlchitryCuPWMVec<const P: usize, const F: u64> {
    clock: Signal<In, Clock<F>>,
    leds: Signal<Out, Bits<8>>,
    local: Signal<Local, Bits<8>>,
    faders: [Fader<F>; 8],
}

impl<const P: usize, const F: u64> Logic for AlchitryCuPWMVec<P, F> {
    #[hdl_gen]
    fn update(&mut self) {
        for i in 0_usize..8_usize {
            self.faders[i].clock.next = self.clock.val();
            self.faders[i].enable.next = true;
        }
        self.local.next = 0x00_u8.into();
        for i in 0_usize..8_usize {
            self.local.next = self.local.val().replace_bit(i, self.faders[i].active.val());
        }
        self.leds.next = self.local.val();
    }
}

impl<const P: usize> Default for AlchitryCuPWMVec<P, {100_000_000}> {
    fn default() -> Self {
        let faders : [Fader<100_000_000>; 8] =
        [
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
            faders
        }
    }
}

#[test]
fn test_pwm_vec_synthesizes() {
    let mut uut : AlchitryCuPWMVec<6, 100_000_000> = AlchitryCuPWMVec::default();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("pwm_cu", &vlog).unwrap();
    rust_hdl_alchitry_cu::synth::generate_bitstream(uut, "pwm_cu");
}
