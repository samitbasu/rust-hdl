use rust_hdl::core::prelude::*;
mod test_common;
use rust_hdl::widgets::prelude::*;
use std::collections::BTreeMap;
use test_common::snore;

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
        self.pwm.clock.next = self.clock.val();
        self.pwm.enable.next = true;

        self.rom.address.next = self.counter.q.val();

        self.pwm.threshold.next = self.rom.data.val();

        self.strobe.enable.next = true;
        self.strobe.clock.next = self.clock.val();

        self.leds.next = 0x00_u8.into();
        if self.pwm.active.val() {
            self.leds.next = 0xFF_u8.into();
        }

        self.counter.clock.next = self.clock.val();
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
    }
}

impl<const P: usize> AlchitryCuPWM<P> {
    fn new(clock_freq: u64) -> Self {
        let rom = (0..256_u32)
            .map(|x| (Bits::<8>::from(x), snore(x)))
            .collect::<BTreeMap<_, _>>();
        Self {
            pwm: PulseWidthModulator::default(),
            clock: rust_hdl::bsp::alchitry_cu::pins::clock(),
            strobe: Strobe::new(clock_freq, 60.0),
            leds: rust_hdl::bsp::alchitry_cu::pins::leds(),
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
    println!("{}", vlog);
    yosys_validate("pwm_cu2", &vlog).unwrap();
    rust_hdl::bsp::alchitry_cu::synth::generate_bitstream(uut, target_path!("alchitry_cu/pwm_cu2"));
}
