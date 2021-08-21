use crate::ok_tools::ok_test_prelude;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::ok_trigger::{TriggerIn, TriggerOut};
use rust_hdl_ok::prelude::*;
use rust_hdl_ok_frontpanel_sys::{OkError, OkHandle};
use rust_hdl_widgets::prelude::DFF;
use std::time::Duration;

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010WireTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>>,
    pub wire_0: WireIn<0x0>,
    pub wire_1: WireIn<0x1>,
    pub o_wire: WireOut<0x20>,
    pub o_wire_1: WireOut<0x21>,
    pub trig: TriggerIn<0x40>,
    pub o_trig: TriggerOut<0x60>,
    pub trig_counter: DFF<Bits<16>>,
}

impl OpalKellyXEM6010WireTest {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            trig_counter: DFF::new(0_u16.into()),
            led: xem_6010_leds(),
            wire_0: Default::default(),
            wire_1: Default::default(),
            o_wire: Default::default(),
            o_wire_1: Default::default(),
            trig: Default::default(),
            ok_host: Default::default(),
            o_trig: Default::default(),
        }
    }
}

impl Logic for OpalKellyXEM6010WireTest {
    #[hdl_gen]
    fn update(&mut self) {
        self.hi.link(&mut self.ok_host.hi);
        self.led.next = bit_cast::<8, 16>(!(self.wire_0.dataout.val() & self.wire_1.dataout.val()));
        self.o_wire.datain.next = self.wire_0.dataout.val();
        //
        self.trig_counter.d.next = self.trig_counter.q.val() + self.trig.trigger.val();
        if self.trig_counter.q.val() == 0x0A_u32 {
            self.o_trig.trigger.next = 0x01_u32.into();
        } else {
            self.o_trig.trigger.next = 0x00_u32.into();
        }
        self.o_wire_1.datain.next = self.trig_counter.q.val();
        // Fan out clock
        self.trig_counter.clk.next = self.ok_host.ti_clk.val();
        self.trig.clk.next = self.ok_host.ti_clk.val();
        self.o_trig.clk.next = self.ok_host.ti_clk.val();
        // Fan out OK1
        self.wire_0.ok1.next = self.ok_host.ok1.val();
        self.wire_1.ok1.next = self.ok_host.ok1.val();
        self.o_wire.ok1.next = self.ok_host.ok1.val();
        self.o_wire_1.ok1.next = self.ok_host.ok1.val();
        self.trig.ok1.next = self.ok_host.ok1.val();
        self.o_trig.ok1.next = self.ok_host.ok1.val();
        // Wire or in OK2
        self.ok_host.ok2.next =
            self.o_wire.ok2.val() | self.o_wire_1.ok2.val() | self.o_trig.ok2.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_wire() {
    let mut uut = OpalKellyXEM6010WireTest::new();
    uut.hi.link_connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_wire");
}

#[test]
fn test_opalkelly_xem_6010_wire_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_wire/top.bit")?;
    hnd.set_wire_in(0x00, 0x45);
    hnd.update_wire_ins();
    for i in 0..12 {
        std::thread::sleep(Duration::from_secs(1));
        let w1 = if i % 2 == 0 { 0xFF } else { 0x00 };
        hnd.set_wire_in(0x01, w1);
        hnd.set_wire_in(0x00, 0x42 + i);
        hnd.activate_trigger_in(0x40, 0)?;
        hnd.update_wire_ins();
        hnd.update_wire_outs();
        assert_eq!(hnd.get_wire_out(0x20), 0x42 + i);
        assert_eq!(hnd.get_wire_out(0x21), i + 1);
        hnd.update_trigger_outs();
        if i == 9 {
            assert!(hnd.is_triggered(0x60, 0xFFFF));
        }
    }
    Ok(())
}
