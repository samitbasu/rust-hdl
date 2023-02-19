use rust_hdl::prelude::*;
use rust_hdl__bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl__ok_core::core::prelude::*;
use rust_hdl__ok_core::test_common::pipe::{test_opalkelly_pipe_ram_runtime, OpalKellyPipeRAMTest};

#[test]
fn test_opalkelly_xem_7010_synth_pipe_ram() {
    let mut uut = OpalKellyPipeRAMTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/pipe_ram"));
    test_opalkelly_pipe_ram_runtime(
        target_path!("xem_7010/pipe_ram/top.bit"),
        env!("XEM7010_SERIAL"),
    )
    .unwrap();
}
