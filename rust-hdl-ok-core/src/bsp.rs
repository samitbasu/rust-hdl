use rust_hdl_core::prelude::*;

use crate::OpalKellyHost;
use crate::OpalKellyHostInterface;

pub trait OpalKellyBSP {
    fn hi() -> OpalKellyHostInterface;
    fn ok_host() -> OpalKellyHost;
    fn leds() -> Signal<Out, Bits<8>>;
    fn clocks() -> Vec<Signal<In, Clock>>;
}
