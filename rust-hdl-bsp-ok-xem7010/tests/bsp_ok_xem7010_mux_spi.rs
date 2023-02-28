use rust_hdl::prelude::*;
use rust_hdl_bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl_lib_ok_core::core::prelude::*;
use rust_hdl_lib_ok_core::test_common::mux_spi::{
    test_opalkelly_mux_spi_runtime, OpalKellySPIMuxTest,
};

#[test]
fn test_opalkelly_xem_7010_mux_spi() {
    let mut uut = OpalKellySPIMuxTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/mux_spi"));
    test_opalkelly_mux_spi_runtime(
        target_path!("xem_7010/mux_spi/top.bit"),
        env!("XEM7010_SERIAL"),
    )
    .unwrap()
}
