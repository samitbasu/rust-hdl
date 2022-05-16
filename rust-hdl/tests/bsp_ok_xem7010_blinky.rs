use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::core::prelude::*;

#[cfg(feature = "frontpanel")]
mod test_common;

#[cfg(feature = "frontpanel")]
use rust_hdl::bsp::ok_xem7010::XEM7010;
#[cfg(feature = "frontpanel")]
use test_common::blinky::OpalKellyBlinky;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_7010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/blinky"));
}
