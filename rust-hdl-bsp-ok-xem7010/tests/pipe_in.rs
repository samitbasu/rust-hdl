use rust_hdl_bsp_ok_xem7010::XEM7010;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_test_ok_common::pipe::{OpalKellyPipeTest, test_opalkelly_pipe_in_runtime};
use rust_hdl_ok_core::prelude::*;
use rust_hdl_core::prelude::*;
use rust_hdl_test_core::target_path;

#[test]
fn test_opalkelly_xem_7010_synth_pipe() {
    let mut uut = OpalKellyPipeTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/pipe_in"));
    test_opalkelly_pipe_in_runtime(target_path!("xem_7010/pipe_in/top.bit")).unwrap();
}
