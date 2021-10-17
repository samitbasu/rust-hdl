use rust_hdl_ok_frontpanel_sys::OkError;

#[test]
fn test_opalkelly_xem_6010_mux_spi() {
    let mut uut = OpalKellySPIMuxTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "xem_6010_mux_spi");
}

#[test]
fn test_opalkelly_xem_6010_mux_spi_runtime() -> Result<(), OkError> {
    opalkelly_mux_spi::test_opalkelly_mux_spi_runtime("xem_6010_mux_spi/top.bit")
}
