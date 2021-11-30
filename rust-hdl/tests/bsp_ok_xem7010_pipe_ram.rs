use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::core::prelude::*;
use rust_hdl::widgets::prelude::*;
mod test_common;

use rust_hdl::bsp::ok_xem7010::pins::{xem_7010_leds, xem_7010_neg_clock, xem_7010_pos_clock};
use rust_hdl::bsp::ok_xem7010::sys_clock::OpalKellySystemClock7;
use rust_hdl::bsp::ok_xem7010::XEM7010;
#[cfg(feature = "frontpanel")]
use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};
#[cfg(feature = "frontpanel")]
use test_common::pipe::*;
#[cfg(feature = "frontpanel")]
use test_common::tools::*;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_7010_synth_pipe_ram() {
    let mut uut = OpalKellyPipeRAMTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/pipe_ram"));
    test_opalkelly_pipe_ram_runtime(target_path!("xem_7010/pipe_ram/top.bit"));
}
