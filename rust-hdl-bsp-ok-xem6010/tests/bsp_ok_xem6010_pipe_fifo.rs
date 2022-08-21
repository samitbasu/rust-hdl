use rust_hdl_ok_core::core::prelude::*;

use rust_hdl::core::prelude::*;
use rust_hdl_bsp_ok_xem6010::xem6010::XEM6010;
use rust_hdl_ok_core::test_common::pipe::{
    test_opalkelly_pipe_fifo_runtime, OpalKellyPipeFIFOTest,
};

#[test]
fn test_opalkelly_xem_6010_synth_pipe_fifo() {
    let mut uut = OpalKellyPipeFIFOTest::new::<XEM6010>();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/pipe_fifo"));
    test_opalkelly_pipe_fifo_runtime(target_path!("xem_6010/pipe_fifo/top.bit")).unwrap();
}
