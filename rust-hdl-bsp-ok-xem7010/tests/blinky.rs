use std::time::Duration;

use rust_hdl_bsp_ok_xem7010::sys_clock::OpalKellySystemClock7;
use rust_hdl_bsp_ok_xem7010::XEM7010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_test_ok_common::prelude::*;
use rust_hdl_widgets::prelude::*;
use rust_hdl_test_core::target_path;

#[test]
fn test_opalkelly_xem_7010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
    XEM7010::synth(uut, target_path!("xem_7010/blinky"));
}
