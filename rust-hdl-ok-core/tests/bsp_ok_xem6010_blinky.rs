use rust_hdl_ok_core::core::prelude::*;

use rust_hdl::core::prelude::*;
use rust_hdl_ok_core::xem6010::XEM6010;

mod test_common;

use test_common::blinky::OpalKellyBlinky;

#[test]
fn test_opalkelly_xem_6010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM6010::synth(uut, target_path!("xem_6010/blinky"));
}
