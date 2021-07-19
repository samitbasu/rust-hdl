use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;
use rust_hdl_core::check_connected::check_connected;
use rust_hdl_synth::yosys_validate;
use std::collections::BTreeMap;
use std::f64::consts::PI;

#[derive(LogicBlock)]
pub struct Fader {
    pub clock: Signal<In, Clock>,
    pub active: Signal<Out, Bit>,
    pub enable: Signal<In, Bit>,
    strobe: Strobe<32>,
    pwm: PulseWidthModulator<6>,
    rom: ROM<Bits<8>, Bits<6>>,
    counter: DFF<Bits<8>>
}

impl Fader {
    pub fn new(phase: u32) -> Self {
        let rom = (0..256_u32)
            .map(|x| (Bits::<8>::from(x), snore(x + phase)))
            .collect::<BTreeMap<_, _>>();
        Self {
            clock: Signal::default(),
            active: Signal::new_with_default(false),
            enable: Signal::default(),
            strobe: Strobe::new(100_000_000, 120),
            pwm: PulseWidthModulator::default(),
            rom: ROM::new(rom),
            counter: DFF::new(Bits::<8>::default())
        }
    }
}

impl Logic for Fader {
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
pub struct AlchitryCuPWMVec<const P: usize> {
    clock: Signal<In, Clock>,
    leds: Signal<Out, Bits<8>>,
    faders: [Fader; 8],
}

impl<const P: usize> Logic for AlchitryCuPWMVec<P> {
    #[hdl_gen]
    fn update(&mut self) {
        for i in 0_usize..8_usize {
            self.faders[i].clock.next = self.clock.val();
            self.faders[i].enable.next = true;
        }
        self.leds.next = 0x00_u8.into();
        for i in 0_usize..8_usize {
            self.leds.next.set_bit(i, self.faders[i].active.val());
        }
        /*for i in 0_usize..8_usize {
            self.leds.next.set_bit(i, self.faders[i].active.val());
        }*/
    }
}

fn snore<const P: usize>(x: u32) -> Bits::<P> {
    let amp = (f64::exp(f64::sin(((x as f64) - 128.0/2.)*PI/128.0))-0.36787944)*108.0;
    let amp = (amp.max(0.0).min(255.0).floor()/255.0 * (1 << P) as f64) as u8;
    amp.into()
}

impl<const P: usize> Default for AlchitryCuPWMVec<P> {
    fn default() -> Self {
        let faders : [Fader; 8] =
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
            faders: faders
        }
    }
}

#[test]
fn test_pwm_vec_synthesizes() {
    let mut uut : AlchitryCuPWMVec<6> = AlchitryCuPWMVec::default();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("pwm_cu", &vlog);
    rust_hdl_alchitry_cu::synth::generate_bitstream(uut, "pwm_cu");
}
