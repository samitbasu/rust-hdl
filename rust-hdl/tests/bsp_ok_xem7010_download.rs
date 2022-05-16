mod test_common;
use rust_hdl::bsp::ok_core::prelude::*;
#[cfg(feature = "frontpanel")]
use rust_hdl::bsp::ok_xem7010::XEM7010;
use rust_hdl::core::prelude::*;
#[cfg(feature = "frontpanel")]
use test_common::download::*;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_7010_synth_download() {
    let mut uut = OpalKellyDownloadFIFOTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/download"));
    test_opalkelly_download_runtime(target_path!("xem_7010/download/top.bit")).unwrap();
}
