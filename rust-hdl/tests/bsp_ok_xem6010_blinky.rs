use rust_hdl::bsp::ok_xem6010::XEM6010;
use rust_hdl::core::prelude::*;
use rust_hdl::bsp::ok_core::prelude::*;

mod test_ok_common;

use bsp_alchitry_cu_pwm_vec_srom::test_common::blinky::OpalKellyBlinky;

#[test]
fn test_opalkelly_xem_6010_synth_blinky() {
    let mut uut = OpalKellyBlinky::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    check_connected(&uut);
    XEM6010::synth(uut, target_path!("xem_6010/blinky"));
}
