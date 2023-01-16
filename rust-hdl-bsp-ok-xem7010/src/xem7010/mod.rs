use rust_hdl::prelude::*;

pub mod ddr_fifo7;
pub mod download;
pub mod mcb_if;
pub mod mig7;
pub mod pins;
pub mod synth;
pub mod sys_clock;

use pins::*;
use rust_hdl_ok_core::core::prelude::*;

#[derive(Clone, Debug)]
pub struct XEM7010 {}

impl OpalKellyBSP for XEM7010 {
    fn hi() -> OpalKellyHostInterface {
        OpalKellyHostInterface::xem_7010()
    }
    fn ok_host() -> OpalKellyHost {
        OpalKellyHost::xem_7010()
    }

    fn leds() -> Signal<Out, Bits<8>> {
        xem_7010_leds()
    }
    fn clocks() -> Vec<Signal<In, Clock>> {
        vec![xem_7010_pos_clock(), xem_7010_neg_clock()]
    }

    fn synth<U: Block>(uut: U, dir: &str) {
        synth::synth_obj(uut, dir)
    }
}
