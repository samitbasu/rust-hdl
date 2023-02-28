use rust_hdl::prelude::*;
use rust_hdl_bsp_ok_xem6010::xem6010::{synth, XEM6010};
use rust_hdl_lib_ok_core::test_common::download::{
    test_opalkelly_download32_runtime, OpalKellyDownload32FIFOTest,
};

#[test]
fn test_opalkelly_xem_6010_synth_download32() {
    let mut uut = OpalKellyDownload32FIFOTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    synth::synth_obj(uut, target_path!("xem_6010/download32"));
    test_opalkelly_download32_runtime(
        target_path!("xem_6010/download32/top.bit"),
        env!("XEM6010_SERIAL"),
    )
    .unwrap();
}
