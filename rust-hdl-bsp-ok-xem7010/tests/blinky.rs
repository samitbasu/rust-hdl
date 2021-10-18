use rust_hdl_bsp_ok_xem7010::sys_clock::OpalKellySystemClock7;
use rust_hdl_bsp_ok_xem7010::XEM7010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_test_ok_common::prelude::*;
use rust_hdl_widgets::prelude::*;
use std::time::Duration;

#[test]
fn test_opalkelly_xem_7010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
    rust_hdl_test_ok_common::ok_tools::synth_obj_7010(uut, "xem_7010_blinky");
}

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
        self.pulser.clock.next = self.clk_100mhz.val();
        self.pulser.enable.next = true;
        if self.pulser.pulse.val() {
            self.led.next = 0xFF_u8.into();
        } else {
            self.led.next = 0x00_u8.into();
        }
    }
}

#[test]
fn test_opalkelly_xem_7010_synth_fast_blinky() {
    let mut uut = OpalKellyFastBlinky::new::<XEM7010>();
    uut.clock_n.connect();
    uut.clock_p.connect();
    uut.connect_all();
    check_connected(&uut);
    println!("{}", generate_verilog(&uut));
    rust_hdl_test_ok_common::ok_tools::synth_obj_7010(uut, "xem_7010_fast_blinky");
}
