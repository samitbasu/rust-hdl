use rust_hdl_ok_core::core::prelude::*;
use rust_hdl::core::prelude::*;
use rust_hdl_bsp_ok_xem7010::xem7010::XEM7010;
use rust_hdl_ok_core::test_common::download::{OpalKellyDownloadFIFOTest, test_opalkelly_download_runtime};

#[test]
fn test_opalkelly_xem_7010_synth_download() {
    let mut uut = OpalKellyDownloadFIFOTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/download"));
    test_opalkelly_download_runtime(target_path!("xem_7010/download/top.bit")).unwrap();
}
