use rust_hdl_bsp_ok_xem6010::XEM6010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_test_ok_common::prelude::*;

#[test]
fn test_opalkelly_xem_6010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
    XEM6010::synth(uut, &target_path("xem_6010_blinky"));
}
