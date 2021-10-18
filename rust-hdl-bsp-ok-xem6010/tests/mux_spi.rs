use rust_hdl_bsp_ok_xem6010::XEM6010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_test_ok_common::prelude::*;

#[test]
fn test_opalkelly_xem_6010_mux_spi() {
    let mut uut = OpalKellySPIMuxTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_bsp_ok_xem6010::synth::synth_obj(uut, "xem_6010_mux_spi");
}

#[test]
fn test_opalkelly_xem_6010_mux_spi_runtime() -> Result<(), OkError> {
    test_opalkelly_mux_spi_runtime("xem_6010_mux_spi/top.bit")
}
