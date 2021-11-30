use rust_hdl::core::prelude::*;
use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::widgets::prelude::*;
mod test_common;

use test_common::pipe::*;
use test_common::tools::*;
use rust_hdl::bsp::ok_xem7010::sys_clock::OpalKellySystemClock7;
use rust_hdl::bsp::ok_xem7010::pins::{xem_7010_pos_clock, xem_7010_leds, xem_7010_neg_clock};
use rust_hdl::bsp::ok_xem7010::XEM7010;
use rust_hdl_ok_frontpanel_sys::{OkError, make_u16_buffer};

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
