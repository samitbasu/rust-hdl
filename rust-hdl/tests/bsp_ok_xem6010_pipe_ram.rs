use rust_hdl::bsp::ok_core::prelude::OpalKellyBSP;
use rust_hdl::bsp::ok_xem6010::XEM6010;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;

mod test_common;
#[cfg(feature = "frontpanel")]
use test_common::pipe::*;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_6010_synth_pipe_ram() {
    let mut uut = OpalKellyPipeRAMTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/pipe_ram"));
    test_opalkelly_pipe_ram_runtime(target_path!("xem_6010/pipe_ram/top.bit"));
}
