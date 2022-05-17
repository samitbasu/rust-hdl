use rust_hdl_ok_core::core::prelude::*;
use rust_hdl::core::prelude::*;

mod test_common;


use rust_hdl_ok_core::xem6010::XEM6010;

use test_common::pipe::*;


#[test]
fn test_opalkelly_xem_6010_synth_pipe() {
    let mut uut = OpalKellyPipeTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/pipe_in"));
    test_opalkelly_pipe_in_runtime(target_path!("xem_6010/pipe_in/top.bit")).unwrap();
}
