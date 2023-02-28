use rust_hdl::prelude::*;
use rust_hdl_bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl_lib_ok_core::core::prelude::*;
use rust_hdl_lib_ok_core::test_common::download::{
    test_opalkelly_download32_runtime, OpalKellyDownload32FIFOTest,
};

#[test]
fn test_opalkelly_xem_7010_synth_download32() {
    let mut uut = OpalKellyDownload32FIFOTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/download32"));
    test_opalkelly_download32_runtime(
        target_path!("xem_7010/download32/top.bit"),
        env!("XEM7010_SERIAL"),
    )
    .unwrap();
}
