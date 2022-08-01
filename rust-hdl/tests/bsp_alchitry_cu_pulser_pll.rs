use rust_hdl::bsp::alchitry_cu::ice_pll::ICE40PLLBlock;
use rust_hdl::bsp::alchitry_cu::synth::generate_bitstream;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::pulser::Pulser;
use std::time::Duration;

const MHZ100: u64 = 100_000_000;
const MHZ25: u64 = 25_000_000;

#[derive(LogicBlock)]
pub struct AlchitryCuPulserPLL {
    pulser: Pulser,
    clock: Signal<In, Clock>,
    leds: Signal<Out, Bits<8>>,
    pll: ICE40PLLBlock<MHZ100, MHZ25>,
}

impl Logic for AlchitryCuPulserPLL {
    #[hdl_gen]
    fn update(&mut self) {
        self.pll.clock_in.next = self.clock.val();
        self.pulser.enable.next = self.pll.locked.val();
        self.pulser.clock.next = self.pll.clock_out.val();
        self.leds.next = 0x00.into();
        if self.pulser.pulse.val() {
            self.leds.next = 0xAA.into();
        }
    }
}

impl Default for AlchitryCuPulserPLL {
    fn default() -> Self {
        let pulser = Pulser::new(MHZ25, 1.0, Duration::from_millis(100));
        Self {
            pulser,
            clock: rust_hdl::bsp::alchitry_cu::pins::clock(),
            leds: rust_hdl::bsp::alchitry_cu::pins::leds(),
            pll: ICE40PLLBlock::default(),
        }
    }
}

#[test]
fn synthesize_alchitry_cu_pulser_with_pll() {
    let uut = AlchitryCuPulserPLL::default();
    generate_bitstream(uut, target_path!("alchitry_cu/pulser_pll"));
}
