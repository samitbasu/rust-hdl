use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::bsp::ok_xem6010::pins::{xem_6010_base_clock, xem_6010_leds};
use rust_hdl::bsp::ok_xem6010::pll::{PLLFreqSynthesis, Spartan6PLLSettings};
use rust_hdl::bsp::ok_xem6010::XEM6010;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

#[derive(LogicBlock)]
pub struct OpalKellyPLLTest {
    pll: PLLFreqSynthesis,
    slow_pulser: Pulser,
    med_pulser: Pulser,
    fast_pulser: Pulser,
    led: Signal<Out, Bits<8>>,
    raw_clock: Signal<In, Clock>,
}

impl Default for OpalKellyPLLTest {
    fn default() -> Self {
        let settings = Spartan6PLLSettings {
            clkin_period_ns: 10.0, // Base clock is 100 MHz
            pll_mult: 10,          // Multiply it up to 1000 MHz
            pll_div: 1,            // The output clock is 1000 MHz
            output_divs: [40, 10, 4, 6, 6, 6],
        };
        Self {
            pll: PLLFreqSynthesis::new(settings),
            slow_pulser: Pulser::new(25_000_000, 1.0, std::time::Duration::from_millis(500)),
            med_pulser: Pulser::new(100_000_000, 1.0, std::time::Duration::from_millis(500)),
            fast_pulser: Pulser::new(250_000_000, 1.0, std::time::Duration::from_millis(500)),
            led: xem_6010_leds(),
            raw_clock: xem_6010_base_clock(),
        }
    }
}

impl Logic for OpalKellyPLLTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.pll.clock_in.next = self.raw_clock.val();
        self.pll.reset.next = false.into();
        self.slow_pulser.enable.next = true;
        self.med_pulser.enable.next = true;
        self.fast_pulser.enable.next = true;
        self.slow_pulser.clock.next = self.pll.clock_out0.val();
        self.med_pulser.clock.next = self.pll.clock_out1.val();
        self.fast_pulser.clock.next = self.pll.clock_out2.val();
        self.led.next = bit_cast::<8, 1>(self.slow_pulser.pulse.val().into())
            | (bit_cast::<8, 1>(self.fast_pulser.pulse.val().into()) << 5_usize)
            | (bit_cast::<8, 1>(self.med_pulser.pulse.val().into()) << 7_usize);
    }
}

#[test]
#[cfg(feature = "frontpanel")]
fn test_opalkelly_pll_synth() {
    let mut uut = OpalKellyPLLTest::default();
    uut.raw_clock.connect();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/pll_test"))
}
