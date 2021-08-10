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
    pub wire_0: OpalKellyWireIn<0x0>,
    pub wire_1: OpalKellyWireIn<0x1>,
    pub o_wire: OpalKellyWireOut<0x20>,
    pub o_wire_1: OpalKellyWireOut<0x21>,
}

impl OpalKellyXEM6010WireTest {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::default(),
            led: xem_6010_leds(),
            wire_0: OpalKellyWireIn::default(),
            wire_1: OpalKellyWireIn::default(),
            o_wire: OpalKellyWireOut::default(),
            o_wire_1: OpalKellyWireOut::default(),
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
        self.led.next = tagged_bit_cast::<MHz48, 8, 16>(!(self.wire_0.dataout.val() &
            self.wire_1.dataout.val())).to_async();
        self.o_wire.datain.next = self.wire_0.dataout.val();
        self.o_wire_1.datain.next = !self.wire_1.dataout.val();
        // Fan out OK1
        self.wire_0.ok1.next = self.ok_host.ok1.val();
        self.wire_1.ok1.next = self.ok_host.ok1.val();
        self.o_wire.ok1.next = self.ok_host.ok1.val();
        self.o_wire_1.ok1.next = self.ok_host.ok1.val();
        // Wire or in OK2
        self.ok_host.ok2.next = self.o_wire.ok2.val() | self.o_wire_1.ok2.val();
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
        let w1 = if i % 2 == 0 {
            0xFF
        } else {
            0x00
        };
        hnd.set_wire_in(0x01, w1);
        hnd.set_wire_in(0x00, 0x42+i);
        hnd.update_wire_ins();
        hnd.update_wire_outs();
        assert_eq!(hnd.get_wire_out(0x20), 0x42+i);
        assert_eq!(hnd.get_wire_out(0x21), !w1);
    }
    Ok(())
}