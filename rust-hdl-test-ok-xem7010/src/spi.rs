use rust_hdl_ok_frontpanel_sys::OkError;

#[test]
fn test_opalkelly_xem_7010_synth_spi() {
    let mut uut = OpalKellySPITest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_7010(uut, "xem_7010_spi");
}

#[test]
fn test_opalkelly_xem_7010_spi_reg_read_runtime() -> Result<(), OkError> {
    spi::test_opalkelly_spi_reg_read_runtime("xem_7010_spi/top.bit")
}

#[test]
fn test_opalkelly_spi_reg_write_xem_7010_runtime() -> Result<(), OkError> {
    spi::test_opalkelly_spi_reg_write_runtime("xem_7010_spi/top.bit")
}

#[test]
fn test_opalkelly_xem_7010_spi_single_conversion_runtime() -> Result<(), OkError> {
    spi::test_opalkelly_spi_single_conversion_runtime("xem_7010_spi/top.bit")
}
