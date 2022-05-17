use rust_hdl::core::prelude::*;
use rust_hdl_ok_core::core::prelude::*;
mod test_common;

use {rust_hdl_ok_core::xem7010::XEM7010, test_common::pipe::*};

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
    test_opalkelly_pipe_in_runtime(target_path!("xem_7010/pipe_in/top.bit")).unwrap();
}
