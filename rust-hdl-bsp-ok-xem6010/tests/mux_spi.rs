use rust_hdl_bsp_ok_xem6010::XEM6010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_test_core::target_path;
use rust_hdl_test_ok_common::prelude::*;

#[test]
fn test_opalkelly_xem_6010_mux_spi() {
    let mut uut = OpalKellySPIMuxTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_bsp_ok_xem6010::synth::synth_obj(uut, target_path!("xem_6010/mux_spi"));
    test_opalkelly_mux_spi_runtime(target_path!("xem_6010/mux_spi/top.bit")).unwrap()
}
