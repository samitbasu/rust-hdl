use rust_hdl::prelude::*;
use rust_hdl_lib_bsp_ok_xem6010::xem6010;
use rust_hdl_lib_bsp_ok_xem6010::xem6010::XEM6010;
use rust_hdl_lib_ok_core::test_common::download::{
    test_opalkelly_download_runtime, OpalKellyDownloadFIFOTest,
};

#[test]
fn test_opalkelly_xem_6010_synth_download() {
    let mut uut = OpalKellyDownloadFIFOTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    xem6010::synth::synth_obj(uut, target_path!("xem_6010/download"));
    test_opalkelly_download_runtime(
        target_path!("xem_6010/download/top.bit"),
        env!("XEM6010_SERIAL"),
    )
    .unwrap()
}
