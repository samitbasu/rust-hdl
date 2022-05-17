use std::time::Duration;

use super::core::prelude::*;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
use pins::*;

pub mod ddr_fifo;
pub mod mcb_if;
pub mod mig;
pub mod ok_download_ddr;
pub mod pins;
pub mod pll;
pub mod synth;

#[derive(Clone, Debug)]
pub struct XEM6010 {}

impl OpalKellyBSP for XEM6010 {
    fn hi() -> OpalKellyHostInterface {
        OpalKellyHostInterface::xem_6010()
    }
    fn ok_host() -> OpalKellyHost {
        OpalKellyHost::xem_6010()
    }

    fn leds() -> Signal<Out, Bits<8>> {
        xem_6010_leds()
    }
    fn clocks() -> Vec<Signal<In, Clock>> {
        vec![xem_6010_base_clock()]
    }

    fn synth<U: Block>(uut: U, dir: &str) {
        synth::synth_obj(uut, dir)
    }
}

#[derive(LogicBlock)]
pub struct OKTest1 {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub led: Signal<Out, Bits<8>>,
    pub pulser: Pulser,
    pub auto_reset: AutoReset,
}

impl OKTest1 {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: OpalKellyHost::xem_6010(),
            led: pins::xem_6010_leds(),
            pulser: Pulser::new(MHZ48, 1.0, Duration::from_millis(500)),
            auto_reset: Default::default(),
        }
    }
}

impl Logic for OKTest1 {
    #[hdl_gen]
    fn update(&mut self) {
        OpalKellyHostInterface::link(&mut self.hi, &mut self.ok_host.hi);
        self.auto_reset.clock.next = self.ok_host.ti_clk.val();
        self.pulser.clock.next = self.ok_host.ti_clk.val();
        self.pulser.reset.next = self.auto_reset.reset.val();
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
    uut.connect_all();
    let vlog = generate_verilog(&uut);
    let _ucf = rust_hdl::toolchain::ise::generate_ucf(&uut);
    yosys_validate("vlog", &vlog).unwrap();
}
