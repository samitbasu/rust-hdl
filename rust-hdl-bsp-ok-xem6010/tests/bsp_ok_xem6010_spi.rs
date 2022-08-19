use rust_hdl::core::prelude::*;
use rust_hdl_bsp_ok_xem6010::xem6010::{synth, XEM6010};
use rust_hdl_ok_core::core::prelude::*;
use rust_hdl_ok_core::test_common::spi::{OpalKellySPITest, test_opalkelly_spi_reg_read_runtime, test_opalkelly_spi_reg_write_runtime, test_opalkelly_spi_single_conversion_runtime};

#[test]
fn test_opalkelly_xem_6010_synth_spi() {
    let mut uut = OpalKellySPITest::new::<XEM6010>();
    uut.connect_all();
    synth::synth_obj(uut, target_path!("xem_6010/spi"));
    test_opalkelly_spi_reg_read_runtime(target_path!("xem_6010/spi/top.bit")).unwrap();
    test_opalkelly_spi_reg_write_runtime(target_path!("xem_6010/spi/top.bit")).unwrap();
    test_opalkelly_spi_single_conversion_runtime(target_path!("xem_6010/spi/top.bit")).unwrap();
}
