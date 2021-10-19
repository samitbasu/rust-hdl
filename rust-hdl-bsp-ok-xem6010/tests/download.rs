use rust_hdl_bsp_ok_xem6010::XEM6010;
use rust_hdl_core::prelude::*;
use rust_hdl_ok_frontpanel_sys::OkError;
use rust_hdl_test_core::target_path;
use rust_hdl_test_ok_common::prelude::*;

#[test]
fn test_opalkelly_xem_6010_synth_download() {
    let mut uut = OpalKellyDownloadFIFOTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_bsp_ok_xem6010::synth::synth_obj(uut, target_path!("xem_6010/download"));
    test_opalkelly_download_runtime(target_path!("xem_6010/download/top.bit")).unwrap()
}
