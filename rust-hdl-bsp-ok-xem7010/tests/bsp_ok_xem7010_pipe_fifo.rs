use rust_hdl_bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl_ok_core::test_common::pipe::{
    test_opalkelly_pipe_fifo_runtime, OpalKellyPipeFIFOTest,
};
use {rust_hdl::core::prelude::*, rust_hdl_ok_core::core::prelude::*};

#[test]
fn test_opalkelly_xem_7010_synth_pipe_fifo() {
    let mut uut = OpalKellyPipeFIFOTest::new::<XEM7010>();
    uut.hi.sig_inout.connect();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/pipe_fifo"));
    test_opalkelly_pipe_fifo_runtime(target_path!("xem_7010/pipe_fifo/top.bit")).unwrap();
}
