use rust_hdl::prelude::*;
use rust_hdl__bsp_ok_xem7010::xem7010::sys_clock::OpalKellySystemClock7;
use rust_hdl__bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl__ok_core::core::prelude::*;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct OpalKellyFastBlinky {
    pub led: Signal<Out, Bits<8>>,
    pub pulser: Pulser,
    pub clock_p: Signal<In, Clock>,
    pub clock_n: Signal<In, Clock>,
    pub clk_100mhz: Signal<Local, Clock>,
    pub clock_div: OpalKellySystemClock7,
}

impl OpalKellyFastBlinky {
    pub fn new<B: OpalKellyBSP>() -> Self {
        let clk = B::clocks();
        Self {
            led: B::leds(),
            pulser: Pulser::new(MHZ100, 1.0, Duration::from_millis(500)),
            clock_p: clk[0].clone(),
            clock_n: clk[1].clone(),
            clk_100mhz: Default::default(),
            clock_div: Default::default(),
        }
    }
}

impl Logic for OpalKellyFastBlinky {
    #[hdl_gen]
    fn update(&mut self) {
        self.clock_div.clock_p.next = self.clock_p.val();
        self.clock_div.clock_n.next = self.clock_n.val();
        self.clk_100mhz.next = self.clock_div.sys_clock.val();
        clock!(self, clk_100mhz, pulser);
        self.pulser.enable.next = true;
        if self.pulser.pulse.val() {
            self.led.next = 0xFF.into();
        } else {
            self.led.next = 0x00.into();
        }
    }
}

#[test]
fn test_fast_blinky_is_synthesizable() {
    let mut uut = OpalKellyFastBlinky::new::<XEM7010>();
    uut.clock_n.connect();
    uut.clock_p.connect();
    uut.connect_all();
    yosys_validate("fast_blinky_7010", &generate_verilog(&uut)).unwrap();
}

#[test]
fn test_opalkelly_xem_7010_synth_fast_blinky() {
    let mut uut = OpalKellyFastBlinky::new::<XEM7010>();
    uut.clock_n.connect();
    uut.clock_p.connect();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/fast_blinky"));
}
