use rust_hdl_core::prelude::*;
use rust_hdl_widgets::prelude::*;
use rust_hdl_ok::*;
use std::time::Duration;

const FRONTPANEL_DIR: &str = "/opt/FrontPanel-Ubuntu16.04LTS-x64-5.2.0/FrontPanelHDL/XEM6010-LX45";

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010Blinky {
    pub hi: okHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>, Async>,
    pub pulser: Pulser<MHz48>,
}

impl OpalKellyXEM6010Blinky {
    pub fn new() -> Self {
        Self {
            hi: okHostInterface::xem_6010(),
            ok_host: OpalKellyHost::default(),
            led: xem_6010_leds(),
            pulser: Pulser::new(1.0, Duration::from_millis(500))
        }
    }
}

impl Logic for OpalKellyXEM6010Blinky {
    #[hdl_gen]
    fn update(&mut self) {
        self.ok_host.hi.sig_in.next = self.hi.sig_in.val();
        self.hi.sig_out.next = self.ok_host.hi.sig_out.val();
        link!(self.hi.sig_inout, self.ok_host.hi.sig_inout);
        link!(self.hi.sig_aa, self.ok_host.hi.sig_aa);
        self.pulser.clock.next = self.ok_host.ti_clk.val();
        self.pulser.enable.next = true.into();
        if self.pulser.pulse.val().any() {
            self.led.next = 0xFF_u8.into();
        } else {
            self.led.next = 0x00_u8.into();
        }
    }
}

#[cfg(test)]
fn synth_obj<U: Block>(uut: U, dir: &str) {
    check_connected(&uut);
    let vlog = generate_verilog(&uut);
    println!("{}", vlog);
    let ucf = rust_hdl_ok::ucf_gen::generate_ucf(&uut);
    println!("{}", ucf);
    rust_hdl_synth::yosys_validate("vlog", &vlog).unwrap();
    rust_hdl_ok::synth::generate_bitstream_xem_6010(uut, dir, &[
        "okLibrary.v",
        "okCoreHarness.ngc",
        "okWireIn.ngc",
        "TFIFO64x8a_64x8b.ngc",
        "okWireOut.ngc"
    ], FRONTPANEL_DIR);
}

#[test]
fn test_opalkelly_xem_6010_blinky() {
    let mut uut = OpalKellyXEM6010Blinky::new();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_inout.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    synth_obj(uut, "opalkelly_xem_6010_blinky");
}
