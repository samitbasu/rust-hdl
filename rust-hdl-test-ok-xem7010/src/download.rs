use rust_hdl_ok_frontpanel_sys::OkError;

#[test]
fn test_opalkelly_xem_7010_synth_download32() {
    let mut uut = OpalKellyDownload32FIFOTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_7010(uut, "xem_7010_download32");
}

#[test]
fn test_opalkelly_xem_7010_download32_runtime() -> Result<(), OkError> {
    download::test_opalkelly_download32_runtime("xem_7010_download32/top.bit")
}

#[test]
fn test_opalkelly_xem_7010_synth_download() {
    let mut uut = OpalKellyDownloadFIFOTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_7010(uut, "xem_7010_download");
}

#[test]
fn test_opalkelly_xem_7010_download_runtime() -> Result<(), OkError> {
    download::test_opalkelly_download_runtime("xem_7010_download/top.bit")
}
