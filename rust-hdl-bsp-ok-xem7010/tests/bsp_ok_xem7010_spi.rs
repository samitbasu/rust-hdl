use rust_hdl::core::prelude::*;
use rust_hdl_bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl_ok_core::core::bsp::OpalKellyBSP;
use rust_hdl_ok_core::test_common::spi::{OpalKellySPITest, test_opalkelly_spi_reg_read_runtime, test_opalkelly_spi_reg_write_runtime, test_opalkelly_spi_single_conversion_runtime};


#[test]
fn test_opalkelly_xem_7010_synth_spi() {
    let mut uut = OpalKellySPITest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/spi"));
    test_opalkelly_spi_reg_read_runtime(target_path!("xem_7010/spi/top.bit")).unwrap();
    test_opalkelly_spi_reg_write_runtime(target_path!("xem_7010/spi/top.bit")).unwrap();
    test_opalkelly_spi_single_conversion_runtime(target_path!("xem_7010/spi/top.bit")).unwrap();
}
