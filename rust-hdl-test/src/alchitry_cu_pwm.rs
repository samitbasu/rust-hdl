use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;
use rust_hdl_core::check_connected::check_connected;
use rust_hdl_synth::yosys_validate;
use std::collections::BTreeMap;
use crate::snore::snore;

#[derive(LogicBlock)]
pub struct AlchitryCuPWM<const P: usize, const F: u64> {
    pwm: PulseWidthModulator<P, F>,
    clock: Signal<In, Clock<F>>,
    strobe: Strobe<32, {F}>,
    leds: Signal<Out, Bits<8>>,
    rom: ROM<Bits<8>, Bits<P>>,
    counter: DFF<Bits<8>, F>
}

impl<const P: usize, const F: u64> Logic for AlchitryCuPWM<P, F> {
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

        self.counter.clk.next = self.clock.val();
        self.counter.d.next = self.counter.q.val() + self.strobe.strobe.val();
    }
}

impl<const P: usize> Default for AlchitryCuPWM<P, 100_000_000> {
    fn default() -> Self {
        let rom = (0..256_u32)
            .map(|x| (Bits::<8>::from(x), snore(x)))
            .collect::<BTreeMap<_, _>>();
        Self {
            pwm: PulseWidthModulator::default(),
            clock: rust_hdl_alchitry_cu::pins::clock(),
            strobe: Strobe::new(60.0),
            leds: rust_hdl_alchitry_cu::pins::leds(),
            rom: ROM::new(rom),
            counter: DFF::new(0_u8.into())
        }
    }
}

#[test]
fn test_pwm_synthesizes() {
    let mut uut : AlchitryCuPWM<6, 100_000_000> = AlchitryCuPWM::default();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    yosys_validate("pwm_cu", &vlog).unwrap();
    rust_hdl_alchitry_cu::synth::generate_bitstream(uut, "pwm_cu");
}

