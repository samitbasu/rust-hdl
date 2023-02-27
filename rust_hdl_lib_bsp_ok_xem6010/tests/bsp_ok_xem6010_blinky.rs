use rust_hdl_lib_ok_core::core::prelude::*;

use rust_hdl::prelude::*;
use rust_hdl_lib_bsp_ok_xem6010::xem6010::XEM6010;
use rust_hdl_lib_ok_core::test_common::blinky::OpalKellyBlinky;

#[test]
fn test_opalkelly_xem_6010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/blinky"));
}
