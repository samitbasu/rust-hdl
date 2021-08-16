#![allow(non_camel_case_types)]

use std::time::Duration;

use ok_hi::OpalKellyHostInterface;
use ok_host::OpalKellyHost;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::pulser::Pulser;

pub mod ok_hi;
pub mod ok_host;
pub mod ok_pipe;
pub mod ok_trigger;
pub mod ok_wire;
pub mod pins;
pub mod prelude;
pub mod synth;
pub mod ucf_gen;

const MHZ48: u64 = 48_000_000;

#[derive(LogicBlock)]
pub struct OKTest1 {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>>,
    pub pulser: Pulser<MHZ48>,
}

macro_rules! link {
    ($from: expr, $to: expr) => {};
}

impl OKTest1 {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::default(),
            led: pins::xem_6010_leds(),
            pulser: Pulser::new(1.0, Duration::from_millis(500)),
        }
    }
}

impl Logic for OKTest1 {
    #[hdl_gen]
    fn update(&mut self) {
        link!(self.hi.sig_in, self.ok_host.hi.sig_in);
        link!(self.hi.sig_inout, self.ok_host.hi.sig_inout);
        link!(self.hi.sig_out, self.ok_host.hi.sig_out);
        link!(self.hi.sig_aa, self.ok_host.hi.sig_aa);
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
fn test_ok_host_synthesizable() {
    let mut uut = OKTest1::new();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_inout.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    let ucf = crate::ucf_gen::generate_ucf(&uut);
    println!("{}", ucf);
    rust_hdl_synth::yosys_validate("vlog", &vlog).unwrap();
}