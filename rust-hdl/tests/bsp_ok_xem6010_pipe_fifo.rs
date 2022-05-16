use rust_hdl::bsp::ok_core::prelude::*;
#[cfg(feature = "frontpanel")]
use rust_hdl::bsp::ok_xem6010::XEM6010;
use rust_hdl::core::prelude::*;

mod test_common;

#[cfg(feature = "frontpanel")]
use crate::test_common::pipe::*;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_6010_synth_pipe_fifo() {
    let mut uut = OpalKellyPipeFIFOTest::new::<XEM6010>();
    uut.hi.sig_inout.connect();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/pipe_fifo"));
    test_opalkelly_pipe_fifo_runtime(target_path!("xem_6010/pipe_fifo/top.bit")).unwrap();
}
