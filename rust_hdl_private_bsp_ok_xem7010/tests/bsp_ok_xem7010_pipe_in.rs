use rust_hdl::prelude::*;
use rust_hdl_private_bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl_private_ok_core::core::prelude::*;
use rust_hdl_private_ok_core::test_common::pipe::{
    test_opalkelly_pipe_in_runtime, OpalKellyPipeTest,
};

#[test]
fn test_opalkelly_xem_7010_synthesizes() {
    let mut uut = OpalKellyPipeTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    yosys_validate("pipe_in_7010", &generate_verilog(&uut)).unwrap();
}

#[test]
fn test_opalkelly_xem_7010_synth_pipe() {
    let mut uut = OpalKellyPipeTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/pipe_in"));
    test_opalkelly_pipe_in_runtime(
        target_path!("xem_7010/pipe_in/top.bit"),
        env!("XEM7010_SERIAL"),
    )
    .unwrap();
}
