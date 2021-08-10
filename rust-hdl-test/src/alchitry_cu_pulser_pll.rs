use rust_hdl_alchitry_cu::ice_pll::ICE40PLLBlock;
use rust_hdl_alchitry_cu::pins::Mhz100;
use rust_hdl_alchitry_cu::synth::generate_bitstream;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::pulser::Pulser;
use std::time::Duration;

make_domain!(Mhz25, 25_000_000);

#[derive(LogicBlock)]
pub struct AlchitryCuPulserPLL {
    pulser: Pulser<Mhz25>,
    clock: Signal<In, Clock, Mhz100>,
    leds: Signal<Out, Bits<8>, Async>,
    pll: ICE40PLLBlock<Mhz100, Mhz25>,
}

impl Logic for AlchitryCuPulserPLL {
    #[hdl_gen]
    fn update(&mut self) {
        self.pulser.enable.next = true.into();
        self.pll.clock_in.next = self.clock.val();
        self.pulser.clock.next = self.pll.clock_out.val();
        self.leds.next = 0x00_u8.into();
        if self.pulser.pulse.val().raw() {
            self.leds.next = 0xAA_u8.into();
        }
    }
}

impl Default for AlchitryCuPulserPLL {
    fn default() -> Self {
        let pulser = Pulser::new(1.0, Duration::from_millis(100));
        Self {
            pulser,
            clock: rust_hdl_alchitry_cu::pins::clock(),
            leds: rust_hdl_alchitry_cu::pins::leds(),
            pll: ICE40PLLBlock::default(),
        }
    }
}

#[test]
fn synthesize_alchitry_cu_pulser_with_pll() {
    let uut = AlchitryCuPulserPLL::default();
    generate_bitstream(uut, "pulser_pll");
}
