#[cfg(feature = "frontpanel")]
use rust_hdl::bsp::ok_xem6010::XEM6010;
use rust_hdl::core::prelude::*;

mod test_common;
#[cfg(feature = "frontpanel")]
use test_common::download::*;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_6010_synth_download() {
    let mut uut = OpalKellyDownloadFIFOTest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl::bsp::ok_xem6010::synth::synth_obj(uut, target_path!("xem_6010/download"));
    test_opalkelly_download_runtime(target_path!("xem_6010/download/top.bit")).unwrap()
}
