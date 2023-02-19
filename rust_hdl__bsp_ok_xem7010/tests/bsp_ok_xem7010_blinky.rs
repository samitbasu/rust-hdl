use rust_hdl::prelude::*;
use rust_hdl__bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl__ok_core::core::prelude::*;
use rust_hdl__ok_core::test_common::blinky::OpalKellyBlinky;

#[test]
fn test_opalkelly_xem_7010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/blinky"));
}
