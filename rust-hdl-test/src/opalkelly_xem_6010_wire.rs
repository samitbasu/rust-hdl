use rust_hdl_core::prelude::*;
use rust_hdl_ok::prelude::*;
use rust_hdl_ok_frontpanel_sys::{OkHandle, OkError};
use crate::ok_tools::ok_test_prelude;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010WireTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>, Async>,
    pub wire_0: OpalKellyWireIn<0>,
    pub wire_1: OpalKellyWireIn<1>,
}

impl OpalKellyXEM6010WireTest {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::default(),
            led: xem_6010_leds(),
            wire_0: OpalKellyWireIn::default(),
            wire_1: OpalKellyWireIn::default(),
        }
    }
}

impl Logic for OpalKellyXEM6010WireTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.ok_host.hi.sig_in.next = self.hi.sig_in.val();
        self.hi.sig_out.next = self.ok_host.hi.sig_out.val();
        link!(self.hi.sig_inout, self.ok_host.hi.sig_inout);
        link!(self.hi.sig_aa, self.ok_host.hi.sig_aa);
        self.led.next = tagged_bit_cast::<MHz48, 8, 16>(!(self.wire_0.ep_dataout.val() &
            self.wire_1.ep_dataout.val())).to_async();
        self.wire_0.ok1.next = self.ok_host.ok1.val();
        self.wire_1.ok1.next = self.ok_host.ok1.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_wire() {
    let mut uut = OpalKellyXEM6010WireTest::new();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_inout.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_wire");
}

#[test]
fn test_opalkelly_xem_6010_wire_runtime() -> Result<(), OkError>{
    let hnd = ok_test_prelude("opalkelly_xem_6010_wire/top.bit")?;
    hnd.set_wire_in(0x00, 0x45);
    hnd.update_wire_ins();
    for i in 0..10 {
        std::thread::sleep(Duration::from_secs(1));
        hnd.set_wire_in(0x01, if i % 2 == 0 {
            0xFF
        } else {
            0x00
        });
        hnd.update_wire_ins();
    }
    Ok(())
}