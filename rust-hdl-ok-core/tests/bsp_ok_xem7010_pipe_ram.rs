use rust_hdl::core::prelude::*;
use rust_hdl_ok_core::core::prelude::*;
mod test_common;

use rust_hdl_ok_core::xem7010::XEM7010;

use test_common::pipe::*;

#[test]
fn test_opalkelly_xem_7010_synth_pipe_ram() {
    let mut uut = OpalKellyPipeRAMTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/pipe_ram"));
    test_opalkelly_pipe_ram_runtime(target_path!("xem_7010/pipe_ram/top.bit")).unwrap();
}
