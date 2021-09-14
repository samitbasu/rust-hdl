use std::time::Duration;

use rust_hdl_core::prelude::*;
use rust_hdl_ok::prelude::*;
use rust_hdl_widgets::prelude::*;

#[derive(LogicBlock)]
pub struct OpalKellyXEM7010Blinky {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>>,
    pub pulser: Pulser,
}

impl OpalKellyXEM7010Blinky {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_7010(),
            ok_host: OpalKellyHost::default(),
            led: xem_7010_leds(),
            pulser: Pulser::new(MHZ48, 1.0, Duration::from_millis(500)),
        }
    }
}

impl Logic for OpalKellyXEM7010Blinky {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        self.pulser.clock.next = self.ok_host.ti_clk.val();
        self.pulser.enable.next = true;
        if self.pulser.pulse.val() {
            self.led.next = 0xFF_u8.into();
        } else {
            self.led.next = 0x00_u8.into();
        }
    }
}

#[test]
fn test_opalkelly_xem_7010_blinky() {
    let mut uut = OpalKellyXEM7010Blinky::new();
    uut.hi.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    crate::ok_tools::synth_obj_7010(uut, "opalkelly_xem_7010_blinky");
}
