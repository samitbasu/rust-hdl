use crate::pins::*;
use rust_hdl_core::bits::Bits;
use rust_hdl_core::clock::Clock;
use rust_hdl_core::direction::{In, Out};
use rust_hdl_core::prelude::Signal;
use rust_hdl_ok_core::bsp::OpalKellyBSP;
use rust_hdl_ok_core::ok_hi::OpalKellyHostInterface;
use rust_hdl_ok_core::ok_host::OpalKellyHost;
pub mod pins;
pub mod synth;

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
}
