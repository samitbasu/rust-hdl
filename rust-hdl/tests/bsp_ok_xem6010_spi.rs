use rust_hdl::core::prelude::*;

mod test_common;
use test_common::spi::*;
use rust_hdl::bsp::ok_xem6010::*;

#[test]
fn test_opalkelly_xem_6010_synth_spi() {
    let mut uut = OpalKellySPITest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    synth::synth_obj(uut, target_path!("xem_6010/spi"));
    test_opalkelly_spi_reg_read_runtime(target_path!("xem_6010/spi/top.bit")).unwrap();
    test_opalkelly_spi_reg_write_runtime(target_path!("xem_6010/spi/top.bit")).unwrap();
    test_opalkelly_spi_single_conversion_runtime(target_path!("xem_6010/spi/top.bit")).unwrap();
}
