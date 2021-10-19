use rust_hdl_bsp_ok_xem6010::XEM6010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_core::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_test_core::target_path;
use rust_hdl_test_ok_common::pipe::{test_opalkelly_pipe_ram_runtime, OpalKellyPipeRAMTest};

#[test]
fn test_opalkelly_xem_6010_synth_pipe_ram() {
    let mut uut = OpalKellyPipeRAMTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/pipe_ram"));
    test_opalkelly_pipe_ram_runtime(target_path!("xem_6010/pipe_ram/top.bit"));
}
