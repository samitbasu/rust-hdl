use std::time::Duration;

use rust_hdl_core::prelude::*;
use rust_hdl_ok::prelude::*;
use rust_hdl_widgets::prelude::*;

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010Blinky {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>>,
    pub pulser: Pulser<MHZ48>,
}

impl OpalKellyXEM6010Blinky {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::default(),
            led: xem_6010_leds(),
            pulser: Pulser::new(1.0, Duration::from_millis(500)),
        }
    }
}

impl Logic for OpalKellyXEM6010Blinky {
    #[hdl_gen]
    fn update(&mut self) {
        /*
        self.ok_host.hi.sig_in.next = self.hi.sig_in.val();
        self.hi.sig_out.next = self.ok_host.hi.sig_out.val();
        link!(self.hi.sig_inout, self.ok_host.hi.sig_inout);
        link!(self.hi.sig_aa, self.ok_host.hi.sig_aa);
         */
        link!(self.hi, self.ok_host.hi);
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
fn test_opalkelly_xem_6010_blinky() {
    let mut uut = OpalKellyXEM6010Blinky::new();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_inout.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    //    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_blinky");
}
