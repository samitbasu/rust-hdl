use crate::pins::{xem_6010_leds, xem_7010_leds};
use crate::OpalKellyHost;
use crate::OpalKellyHostInterface;
use rust_hdl_core::prelude::*;

pub trait OpalKellyBSP {
    fn hi() -> OpalKellyHostInterface;
    fn ok_host() -> OpalKellyHost;
    fn leds() -> Signal<Out, Bits<8>>;
}

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
}

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
}
