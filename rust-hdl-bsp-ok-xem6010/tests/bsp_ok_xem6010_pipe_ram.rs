use rust_hdl::prelude::*;
use rust_hdl_bsp_ok_xem6010::xem6010::XEM6010;
use rust_hdl_ok_core::core::prelude::OpalKellyBSP;
use rust_hdl_ok_core::test_common::pipe::{test_opalkelly_pipe_ram_runtime, OpalKellyPipeRAMTest};

#[test]
fn test_opalkelly_xem_6010_synth_pipe_ram() {
    let mut uut = OpalKellyPipeRAMTest::new::<XEM6010>();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/pipe_ram"));
    test_opalkelly_pipe_ram_runtime(
        target_path!("xem_6010/pipe_ram/top.bit"),
        env!("XEM6010_SERIAL"),
    )
    .unwrap();
}
