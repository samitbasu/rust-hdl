use rust_hdl::prelude::*;
use rust_hdl_bsp_ok_xem6010::xem6010::XEM6010;
use rust_hdl_lib_ok_core::core::prelude::*;
use rust_hdl_lib_ok_core::test_common::pipe::{test_opalkelly_pipe_in_runtime, OpalKellyPipeTest};

#[test]
fn test_opalkelly_xem_6010_synth_pipe() {
    let mut uut = OpalKellyPipeTest::new::<XEM6010>();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/pipe_in"));
    test_opalkelly_pipe_in_runtime(
        target_path!("xem_6010/pipe_in/top.bit"),
        env!("XEM6010_SERIAL"),
    )
    .unwrap();
}
