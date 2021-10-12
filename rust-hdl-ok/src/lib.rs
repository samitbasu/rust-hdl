#![allow(non_camel_case_types)]

use std::time::Duration;

use ok_hi::OpalKellyHostInterface;
use ok_host::OpalKellyHost;
use rust_hdl_core::prelude::*;
use rust_hdl_widgets::pulser::Pulser;

pub mod bsp;
pub mod ddr_fifo;
pub mod ddr_fifo7;
pub mod mcb_if;
pub mod mig;
pub mod mig7;
pub mod ok_download;
pub mod ok_download_ddr;
pub mod ok_download_ddr7;
pub mod ok_hi;
pub mod ok_host;
pub mod ok_pipe;
pub mod ok_sys_clock7;
pub mod ok_trigger;
pub mod ok_wire;
pub mod pins;
pub mod prelude;
pub mod spi;
pub mod synth_6010;
pub mod synth_7010;
pub mod synth_common;
pub mod ucf_gen;
pub mod xdc_gen;

const MHZ48: u64 = 48_000_000;

#[derive(LogicBlock)]
pub struct OKTest1 {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>>,
    pub pulser: Pulser,
}

impl OKTest1 {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::xem_6010(),
            led: pins::xem_6010_leds(),
            pulser: Pulser::new(MHZ48, 1.0, Duration::from_millis(500)),
        }
    }
}

impl Logic for OKTest1 {
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
